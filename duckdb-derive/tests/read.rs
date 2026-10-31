use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, ToRow, Table)]
struct User {
    #[duckdb(primary_key)]
    id: i32,
    #[duckdb(unique)]
    email: String,
    #[duckdb(default = "0", check = "score >= 0")]
    score: i32,
}

fn user(id: i32, email: &str, score: i32) -> User {
    User {
        id,
        email: email.into(),
        score,
    }
}

fn db() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    c.create_table::<User>().unwrap();
    c
}

#[test]
fn find_and_select_all() {
    let c = db();
    c.insert_batch(&[user(1, "a@x", 1), user(2, "b@x", 2)])
        .unwrap();
    assert_eq!(c.find::<User, _>([2]).unwrap(), Some(user(2, "b@x", 2)));
    assert_eq!(c.find::<User, _>([9]).unwrap(), None);
    assert_eq!(c.select_all::<User>().unwrap().len(), 2);
}

#[test]
fn query_one_and_optional() {
    let c = db();
    c.insert(&user(1, "a@x", 1)).unwrap();
    let sql = "FROM user ORDER by id";
    assert_eq!(c.query_one::<User, _>(sql, []).unwrap(), user(1, "a@x", 1));
    assert!(
        c.query_optional::<User, _>("FROM user WHERE id = 9", [])
            .unwrap()
            .is_none()
    );
    assert!(
        c.query_one::<User, _>("FROM user WHERE id = 9", [])
            .is_err()
    );
}

#[test]
fn constraints_test() {
    let sql = User::create_table_sql();
    assert!(sql.contains("UNIQUE"));
    assert!(sql.contains("DEFAULT 0"));
    assert!(sql.contains("CHECK (score >= 0)"));

    let c = db();
    c.insert(&user(1, "a@x", 1)).unwrap();
    assert!(c.insert(&user(2, "a@x", 1)).is_err());
    assert!(c.insert(&user(3, "c@x", -1)).is_err());
}
