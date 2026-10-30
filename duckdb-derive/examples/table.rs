use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Ducks {
    id: i32,
    name: String,
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
        },
        Ducks {
            id: 2,
            name: "Donald".into(),
        },
        Ducks {
            id: 3,
            name: "Darkwing Duck".into(),
        },
    ])?;

    println!("db saved with path: {}", db_path);

    Ok(())
}
