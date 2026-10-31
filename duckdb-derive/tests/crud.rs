use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, ToRow, Table)]
struct Duck {
    #[duckdb(primary_key)]
    id: i32,
    name: String,
}

#[derive(Debug, PartialEq, FromRow, ToRow, Table)]
struct Membership {
    #[duckdb(primary_key)]
    user_id: i32,
    #[duckdb(primary_key)]
    group_id: i32,
    role: String,
}

fn duck(id: i32, name: &str) -> Duck {
    Duck {
        id,
        name: name.into(),
    }
}

#[test]
fn upsert_inserts_then_updates() {
    let c = Connection::open_in_memory().unwrap();
    c.create_table::<Duck>().unwrap();
    c.upsert(&duck(1, "a")).unwrap();
    c.upsert(&duck(1, "b")).unwrap();
    let all: Vec<Duck> = c.query_as("FROM duck", []).unwrap();
    assert_eq!(all, vec![duck(1, "b")]);
}

#[test]
fn insert_or_ignore_keeps_existing() {
    let c = Connection::open_in_memory().unwrap();
    c.create_table::<Duck>().unwrap();
    assert_eq!(c.insert_or_ignore(&duck(1, "a")).unwrap(), 1);
    assert_eq!(c.insert_or_ignore(&duck(1, "b")).unwrap(), 0);
    let all: Vec<Duck> = c.query_as("FROM duck", []).unwrap();
    assert_eq!(all, vec![duck(1, "a")]);
}

#[test]
fn update_and_delete() {
    let c = Connection::open_in_memory().unwrap();
    c.create_table::<Duck>().unwrap();
    c.insert(&duck(1, "a")).unwrap();
    assert_eq!(c.update(&duck(1, "z")).unwrap(), 1);
    assert_eq!(c.update(&duck(2, "x")).unwrap(), 0);
    assert_eq!(c.delete(&duck(1, "ignored")).unwrap(), 1);
    let all: Vec<Duck> = c.query_as("FROM duck", []).unwrap();
    assert!(all.is_empty());
}

#[test]
fn insert_returning_roundtrips() {
    let c = Connection::open_in_memory().unwrap();
    c.create_table::<Duck>().unwrap();
    assert_eq!(c.insert_returning(&duck(7, "q")).unwrap(), duck(7, "q"));
}

#[test]
fn composite_key() {
    assert!(Membership::create_table_sql().contains(r#"PRIMARY KEY ("user_id", "group_id")"#));
    let c = Connection::open_in_memory().unwrap();
    c.create_table::<Membership>().unwrap();
    let m = |u, g, r: &str| Membership {
        user_id: u,
        group_id: g,
        role: r.into(),
    };
    c.upsert(&m(1, 1, "a")).unwrap();
    c.upsert(&m(1, 2, "b")).unwrap();
    c.upsert(&m(1, 1, "c")).unwrap();
    assert_eq!(c.update(&m(1, 2, "d")).unwrap(), 1);
    assert_eq!(c.delete(&m(1, 1, "")).unwrap(), 1);
    let all: Vec<Membership> = c.query_as("FROM membership", []).unwrap();
    assert_eq!(all, vec![m(1, 2, "d")]);
}
