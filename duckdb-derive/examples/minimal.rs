//! Small `duckdb-derive` example.
//!
//! Run with `cargo run --example minimal`.

use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, FromRow, Table, ToRow)]
struct Duck {
    #[duckdb(primary_key)]
    id: i32,
    name: String,
}

fn main() -> duckdb::Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.create_table::<Duck>()?;
    conn.insert(&Duck {
        id: 1,
        name: "Donald Duck".into(),
    })?;
    conn.insert(&Duck {
        id: 2,
        name: "Scrooge McDuck".into(),
    })?;

    let ducks: Vec<Duck> = conn.query_as("FROM duck ORDER BY id", [])?;
    for duck in ducks {
        println!("{}) {}", duck.id, duck.name);
    }

    Ok(())
}
