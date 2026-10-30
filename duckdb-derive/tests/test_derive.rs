use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Ducks {
    id: i32,
    name: String,
}

#[test]
fn test_full_cycle() -> duckdb::Result<()> {
    let conn = Connection::open_in_memory()?;

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

    let ducks: Vec<Ducks> = conn.query_as("SELECT id, name FROM ducks ORDER BY id", [])?;

    assert_eq!(ducks.len(), 3);
    Ok(())
}
