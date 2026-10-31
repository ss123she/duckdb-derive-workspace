#![forbid(unsafe_code)]

//! duckdb-derive is an ergonomic derive-macros for duckdb in Rust.
//!
//! ### The Official DuckDB Example rewritten with `duckdb-derive`:
//!
//! ```rust
//! use duckdb::Connection;
//! use duckdb_derive::{DuckDbExt, FromRow, Table, ToRow};
//!
//! #[derive(Debug, PartialEq, FromRow, Table, ToRow)]
//! struct Person {
//!     #[duckdb(primary_key)]
//!     id: i32,
//!     name: String,
//!     data: Option<Vec<u8>>,
//! }
//!
//! # fn main() -> duckdb::Result<()> {
//! let conn = Connection::open_in_memory()?;
//!
//! // Creates: id INTEGER PRIMARY KEY, name TEXT, data BLOB
//! conn.create_table::<Person>()?;
//!
//! let me = Person {
//!     id: 1,
//!     name: "Steven".into(),
//!     data: None,
//! };
//!
//! // Insert without raw SQL or manual params![] binding
//! conn.insert(&me)?;
//!
//! // Query into typed Person structs (no manual row.get(0/1/2))
//! let people: Vec<Person> = conn.query_as("SELECT id, name, data FROM person", [])?;
//!
//! for person in &people {
//!     println!("Found person: {:?}", person);
//! }
//!
//! assert_eq!(people.len(), 1);
//! assert_eq!(people[0].name, "Steven");
//! # Ok(())
//! # }
//! ```

mod column_type;
mod ext;
mod sql;
mod traits;

pub use column_type::ColumnType;
pub use duckdb_derive_macros::*;
pub use ext::DuckDbExt;
pub use traits::{FromRow, Table, ToRow};

/// Quotes an SQL identifier, escaping embedded `"`.
#[doc(hidden)]
pub fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
