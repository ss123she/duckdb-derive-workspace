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

#[cfg(feature = "extra-types")]
#[test]
fn test_extra_types() -> duckdb::Result<()> {
    use chrono::{DateTime, NaiveDate, Utc};
    use rust_decimal::Decimal;
    use serde_json::json;
    use uuid::Uuid;

    #[derive(Debug, PartialEq, FromRow, Table, ToRow)]
    struct Order {
        #[duckdb(primary_key)]
        id: Uuid,
        order_date: NaiveDate,
        created_at: DateTime<Utc>,
        amount: Decimal,
        payload: serde_json::Value,
    }

    let conn = Connection::open_in_memory()?;
    conn.create_table::<Order>()?;

    let id = Uuid::from_bytes([
        0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
        0xd8,
    ]);
    let order_date = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
    let created_at = DateTime::from_timestamp(1761858000, 0).unwrap();
    let amount = Decimal::new(12345, 2); // 123.45
    let payload = json!({ "status": "shipped", "tags": [1, 2, 3] });

    let order = Order {
        id,
        order_date,
        created_at,
        amount,
        payload: payload.clone(),
    };

    conn.insert_batch(&[order])?;

    let rows: Vec<Order> = conn.query_as("FROM \"order\"", [])?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, id);
    assert_eq!(rows[0].order_date, order_date);
    assert_eq!(rows[0].created_at, created_at);
    assert_eq!(rows[0].amount, amount);
    assert_eq!(rows[0].payload, payload);

    Ok(())
}

#[test]
fn test_nullable_and_single_insert() -> duckdb::Result<()> {
    #[derive(Debug, PartialEq, FromRow, Table, ToRow)]
    #[duckdb(table = "custom_users")]
    struct User {
        #[duckdb(primary_key)]
        id: i64,
        name: String,
        bio: Option<String>,
    }

    let conn = Connection::open_in_memory()?;
    conn.create_table::<User>()?;

    let u1 = User {
        id: 1,
        name: "Alice".into(),
        bio: Some("Chef".into()),
    };

    let u2 = User {
        id: 2,
        name: "Bob".into(),
        bio: None,
    };

    conn.insert(&u1)?;
    conn.insert(&u2)?;

    let users: Vec<User> =
        conn.query_as("SELECT id, name, bio FROM custom_users ORDER BY id", [])?;
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].bio, Some("Chef".into()));
    assert_eq!(users[1].bio, None);

    Ok(())
}
