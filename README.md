# duckdb-derive

[![Crates.io](https://img.shields.io/crates/v/duckdb-derive.svg)](https://crates.io/crates/duckdb-derive)
[![docs.rs](https://docs.rs/duckdb-derive/badge.svg)](https://docs.rs/duckdb-derive)
[![Downloads](https://img.shields.io/crates/d/duckdb-derive.svg)](https://crates.io/crates/duckdb-derive)
[![Rust](https://img.shields.io/badge/rust-1.85.0%2B-blue.svg?logo=rust)](https://www.rust-lang.org)
[![CI](https://github.com/ss123she/duckdb-derive-workspace/actions/workflows/ci.yml/badge.svg)](https://github.com/ss123she/duckdb-derive-workspace/actions)
[![License](https://img.shields.io/crates/l/duckdb-derive.svg)](#license)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Ergonomic derive macros and extensions for [duckdb-rs](https://github.com/duckdb/duckdb-rs).

- Eliminate manual row parsing (`row.get(0)`, `row.get(1)`)
- Generate `CREATE TABLE` DDL directly from struct definitions
- Infer `NOT NULL` constraints for non-`Option<T>` fields
- Bulk insert via DuckDB's native columnar `Appender`
- No ORM, no query builder. Just derives and a small `Connection` extension trait

---

## Quickstart

Add `duckdb` and `duckdb-derive` to your `Cargo.toml`:

```shell
cargo add duckdb
cargo add duckdb-derive
```

### The Official DuckDB Example rewritten with `duckdb-derive`:

```rust
use duckdb::Connection;
use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};

#[derive(Debug, PartialEq, FromRow, Table, ToRow)]
struct Duck {
    #[duckdb(primary_key)]
    id: i32,
    name: String,
}

fn main() -> duckdb::Result<()> {
    let conn = Connection::open_in_memory()?;

    // Creates: CREATE TABLE IF NOT EXISTS "duck" ("id" INTEGER PRIMARY KEY, "name" TEXT NOT NULL);
    conn.create_table::<Duck>()?;

    // Bulk insertion via Appender
    conn.insert_batch(&[
        Duck { id: 1, name: "Donald Duck".into() },
        Duck { id: 2, name: "Scrooge McDuck".into() },
    ])?;

    // Single insert
    conn.insert(&Duck { id: 3, name: "Darkwing Duck".into() })?;

    // Query into typed structs (columns matched by name)
    let ducks: Vec<Duck> = conn.query_as("FROM duck ORDER BY id", [])?;
    for duck in ducks {
        println!("{}) {}", duck.id, duck.name);
    }

    Ok(())
}
```

---

## Examples

- [`minimal.rs`](https://github.com/ss123she/duckdb-derive-workspace/blob/main/duckdb-derive/examples/minimal.rs): create a table, insert, query
- [`table.rs`](https://github.com/ss123she/duckdb-derive-workspace/blob/main/duckdb-derive/examples/table.rs): full tour (upsert, update, delete, find, composite keys)

Run one with `cargo run --example minimal`.

---

## Extension methods

All methods live on `DuckDbExt`, implemented for `duckdb::Connection`.

| Method | Does |
| :--- | :--- |
| `create_table::<T>()` | `CREATE TABLE IF NOT EXISTS` |
| `insert(&t)` | `INSERT` one row (columns named explicitly) |
| `insert_batch(&[t])` | `INSERT` many rows via `Appender` |
| `insert_or_ignore(&t)` | `INSERT ... ON CONFLICT DO NOTHING` |
| `insert_returning(&t)` | `INSERT ... RETURNING *`, read back as `T` |
| `upsert(&t)` | `INSERT ... ON CONFLICT DO UPDATE` |
| `update(&t)` | `UPDATE` non-key columns by primary key |
| `delete(&t)` | `DELETE` by primary key |
| `find::<T, _>(key)` | `SELECT` one row by primary key |
| `select_all::<T>()` | `SELECT *` every row |
| `query_as::<T, _>(sql, params)` | Run a query, collect `Vec<T>` |
| `query_one` / `query_optional` | First row, as `T` / `Option<T>` |

`upsert`, `update`, `delete` and `find` need a `#[duckdb(primary_key)]` field. Mark several fields with `primary_key` for a composite key, and pass the key values in field order to `find`.

---

## Field & Struct Attributes

Customize behavior with `#[duckdb(...)]`:

```rust
#[derive(Debug, FromRow, Table, ToRow)]
#[duckdb(table = "duck_registry")] // Custom name (default: struct name lowercased)
struct Duck {
    #[duckdb(primary_key)]
    id: i32,

    #[duckdb(rename = "full_name")]
    name: String,

    #[duckdb(foreign_key = "nest(id)")]
    nest_id: i32,

    // Override SQL type (e.g., custom precision)
    #[duckdb(sql_type = "DECIMAL(12, 2)")]
    wealth: f64,

    // Append arbitrary SQL constraints
    #[duckdb(extra = "CHECK (age >= 0)")]
    age: u8,

    // Nullable in SQL because of Option<T>
    bio: Option<String>,

    // Excluded from DDL, INSERTs, and SELECT mapping
    #[duckdb(skip)]
    debug_calc: usize,
}
```

| Attribute | Target | Description |
| :--- | :--- | :--- |
| `table = "name"` | Struct | Override table name |
| `primary_key` | Field | Mark as `PRIMARY KEY` |
| `rename = "col"` | Field | Custom column name |
| `foreign_key = "tbl(col)"` | Field | Add `REFERENCES tbl(col)` |
| `sql_type = "TYPE"` | Field | Override SQL type |
| `extra = "SQL"` | Field | Append any other raw SQL |
| `skip` | Field | Skip field (uses `Default` when reading) |
| `unique` | Field | Add `UNIQUE` |
| `default = "SQL"` | Field | Add `DEFAULT SQL` (raw SQL: strings need inner quotes, `"'new'"`) |
| `check = "SQL"` | Field | Add `CHECK (SQL)` |

---

## Supported Types

| Rust | SQL |
| :--- | :--- |
| `i8`..`i64`, `u8`..`u64` | `TINYINT`..`BIGINT`, `UTINYINT`..`UBIGINT` |
| `f32`, `f64` | `FLOAT`, `DOUBLE` |
| `bool`, `String`, `Vec<u8>` | `BOOLEAN`, `TEXT`, `BLOB` |
| `Option<T>` | Nullable (non-`Option` types default to `NOT NULL`) |

### Optional feature flags

| Feature | Adds |
| :--- | :--- |
| `chrono` | `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>` |
| `uuid` | `Uuid` -> `UUID` |
| `rust_decimal` | `Decimal` -> `DECIMAL` (DuckDB default `DECIMAL(18,3)`; use `sql_type` for other precision) |
| `serde_json` | `serde_json::Value` -> `JSON` |
| `extra-types` | all of the above |
| `bundled` | build DuckDB from source (forwards to `duckdb/bundled`) |
---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/ss123she/duckdb-derive-workspace/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/ss123she/duckdb-derive-workspace/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
