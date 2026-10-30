use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow};

#[derive(Debug, PartialEq, FromRow)]
struct Duck {
    id: i32,
    name: String,
}

#[test]
fn test_from_row() -> duckdb::Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute_batch(
        "CREATE TABLE ducks (id INTEGER, name TEXT);
         INSERT INTO ducks VALUES (1, 'Scrooge'), (2, 'McDuck');",
    )?;

    let ducks: Vec<Duck> = conn.query_as("SELECT id, name FROM ducks ORDER BY id", [])?;

    assert_eq!(ducks.len(), 2);
    assert_eq!(
        ducks[0],
        Duck {
            id: 1,
            name: "Scrooge".into()
        }
    );
    assert_eq!(
        ducks[1],
        Duck {
            id: 2,
            name: "McDuck".into()
        }
    );

    println!("Success! Ducks: {:?}", ducks);
    Ok(())
}
