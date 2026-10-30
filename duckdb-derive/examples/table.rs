use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Ducks {
    #[duckdb(primary_key)]
    id: i32,
    #[duckdb(sql_type = "TEXT", extra = "NOT NULL CHECK (length(name) > 0)")]
    name: String,

    age: u8,
    wealth: f64,
    can_fly: bool,
    bio: Option<String>,
}

fn main() -> duckdb::Result<()> {
    let db_path = "target/table.duckdb";

    let _ = std::fs::remove_file(db_path);

    let conn = Connection::open(db_path)?;

    conn.create_table::<Ducks>()?;

    conn.insert_batch(&[
        Ducks {
            id: 1,
            name: "Scrooge".into(),
            age: 75,
            wealth: 999_999_999.99,
            can_fly: false,
            bio: Some("Richest duck in the world".into()),
        },
        Ducks {
            id: 2,
            name: "Donald".into(),
            age: 92,
            wealth: 1.50,
            can_fly: true,
            bio: None,
        },
        Ducks {
            id: 3,
            name: "Darkwing Duck".into(),
            age: 35,
            wealth: 250.00,
            can_fly: false,
            bio: Some("I am the terror that flaps in the night".into()),
        },
    ])?;

    println!("db saved with path: {}", db_path);

    Ok(())
}
