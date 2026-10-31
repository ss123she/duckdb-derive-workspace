//! `duckdb-derive` example of schema, CRUD, queries and a composite key.
//!
//! Run with `cargo run --example table`.

use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Duck {
    #[duckdb(primary_key)]
    id: i32,
    name: String,
}

/// Several `primary_key` fields make a composite key.
#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Enrollment {
    #[duckdb(primary_key)]
    duck_id: i32,
    #[duckdb(primary_key)]
    course_id: i32,
    #[duckdb(default = "0", check = "grade >= 0")]
    grade: i32,
}

fn duck(id: i32, name: &str) -> Duck {
    Duck {
        id,
        name: name.into(),
    }
}

fn main() -> duckdb::Result<()> {
    let conn = Connection::open_in_memory()?;

    // CREATE TABLE IF NOT EXISTS "duck" ("id" INTEGER PRIMARY KEY, "name" TEXT NOT NULL);
    conn.create_table::<Duck>()?;
    println!("{}", Duck::create_table_sql());

    // Bulk insert via the Appender, then a single insert.
    conn.insert_batch(&[duck(1, "Donald Duck"), duck(2, "Scrooge McDuck")])?;
    conn.insert(&duck(3, "Darkwing Duck"))?;

    // Existing key: ignored (0) or updated.
    assert_eq!(conn.insert_or_ignore(&duck(3, "Ignored"))?, 0);
    conn.upsert(&duck(3, "Darkwing"))?;

    // Update columns by key.
    assert_eq!(conn.update(&duck(1, "Donald"))?, 1);

    // Insert and read the stored row back.
    let huey = conn.insert_returning(&duck(4, "Huey"))?;
    assert_eq!(huey, duck(4, "Huey"));

    // Look up by key.
    assert_eq!(conn.find::<Duck, _>([1])?, Some(duck(1, "Donald")));
    assert_eq!(conn.find::<Duck, _>([99])?, None);

    // Delete by key (only the key fields are used).
    assert_eq!(conn.delete(&duck(2, ""))?, 1);

    // Typed queries; columns are matched by name.
    let ducks: Vec<Duck> = conn.query_as("FROM duck ORDER BY id", [])?;
    for d in &ducks {
        println!("{}) {}", d.id, d.name);
    }
    assert_eq!(
        ducks,
        vec![duck(1, "Donald"), duck(3, "Darkwing"), duck(4, "Huey")]
    );

    let first: Duck = conn.query_one("FROM duck ORDER BY id", [])?;
    assert_eq!(first.id, 1);
    let missing: Option<Duck> = conn.query_optional("FROM duck WHERE id = ?", [99])?;
    assert!(missing.is_none());
    assert_eq!(conn.select_all::<Duck>()?.len(), 3);

    // Composite key: pass the key values in field order.
    conn.create_table::<Enrollment>()?;
    conn.insert(&Enrollment {
        duck_id: 1,
        course_id: 10,
        grade: 5,
    })?;
    conn.upsert(&Enrollment {
        duck_id: 1,
        course_id: 10,
        grade: 9,
    })?;
    let e = conn
        .find::<Enrollment, _>([1, 10])?
        .expect("enrollment exists");
    assert_eq!(e.grade, 9);

    println!("ok");
    Ok(())
}
