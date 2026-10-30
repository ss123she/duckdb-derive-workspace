use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow};
use duckdb_derive_macros::Table;

#[derive(Debug, PartialEq, FromRow, Table)]
struct Ducks {
    id: i32,
    name: String,
}

#[test]
fn test_from_row() -> duckdb::Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.create_table::<Ducks>()?;

    conn.execute_batch("INSERT INTO ducks VALUES (1, 'Scrooge'), (2, 'McDuck');")?;

    let ducks: Vec<Ducks> = conn.query_as("SELECT id, name FROM ducks ORDER BY id", [])?;

    assert_eq!(ducks.len(), 2);
    assert_eq!(
        ducks[0],
        Ducks {
            id: 1,
            name: "Scrooge".into()
        }
    );
    assert_eq!(
        ducks[1],
        Ducks {
            id: 2,
            name: "McDuck".into()
        }
    );

    println!("Success! Ducks: {:?}", ducks);
    Ok(())
}
