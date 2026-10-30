use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Department {
    #[duckdb(primary_key)]
    id: i32,
    name: String,
}

#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Employee {
    #[duckdb(primary_key)]
    id: i32,

    #[duckdb(rename = "full_name")]
    name: String,

    #[duckdb(foreign_key = "department(id)")]
    dept_id: i32,

    #[duckdb(skip)]
    debug_calc: i32,
}

#[test]
fn full_test() -> duckdb::Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.create_table::<Department>()?;
    conn.create_table::<Employee>()?;

    conn.insert_batch(&[
        Department {
            id: 1,
            name: "Cartoons".into(),
        },
        Department {
            id: 2,
            name: "Treasure Hunting".into(),
        },
    ])?;

    conn.insert_batch(&[
        Employee {
            id: 101,
            name: "Donald Duck".into(),
            dept_id: 1,
            debug_calc: 10,
        },
        Employee {
            id: 102,
            name: "Scrooge McDuck".into(),
            dept_id: 2,
            debug_calc: 20,
        },
    ])?;

    let employees: Vec<Employee> = conn.query_as(
        "SELECT id, full_name, dept_id FROM employee ORDER BY id",
        [],
    )?;

    assert_eq!(employees.len(), 2);
    assert_eq!(employees[0].name, "Donald Duck");
    assert_eq!(employees[0].dept_id, 1);
    assert_eq!(employees[1].name, "Scrooge McDuck");
    assert_eq!(employees[1].dept_id, 2);
    assert_eq!(employees[0].debug_calc, 0);
    Ok(())
}

#[test]
fn test_transaction() -> duckdb::Result<()> {
    let mut conn = Connection::open_in_memory()?;
    let tx = conn.transaction()?;

    tx.create_table::<Department>()?;
    tx.insert_batch(&[Department {
        id: 1,
        name: "Cartoons".into(),
    }])?;

    let res: Vec<Department> = tx.query_as("FROM department", [])?;
    assert_eq!(res.len(), 1);

    tx.commit()
}
