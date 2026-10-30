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

pub use duckdb_derive_macros::*;

pub trait FromRow: Sized {
    type Indices;
    fn init_indices(stmt: &duckdb::Statement<'_>) -> Result<Self::Indices, duckdb::Error>;
    fn from_row(row: &duckdb::Row<'_>, indices: &Self::Indices) -> Result<Self, duckdb::Error>;
}

pub trait ToRow {
    const COLUMN_COUNT: usize;
    const PLACEHOLDERS: &'static str;

    fn to_params(&self) -> Vec<&dyn duckdb::ToSql>;
}

pub trait Table {
    const TABLE_NAME: &'static str;
    fn create_table_sql() -> String;
}

pub trait DuckDbExt {
    fn query_as<'a, T: FromRow, P: duckdb::Params>(
        &'a self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, duckdb::Error>;
    fn create_table<T: Table>(&self) -> Result<usize, duckdb::Error>;
    fn insert<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error>;
    fn insert_batch<T: Table + ToRow>(&self, items: &[T]) -> Result<(), duckdb::Error>;
}

impl DuckDbExt for duckdb::Connection {
    fn query_as<'a, T: FromRow, P: duckdb::Params>(
        &'a self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, duckdb::Error> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.query(params)?;
        let indices = T::init_indices(rows.as_ref().expect("statement executed"))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(T::from_row(row, &indices)?);
        }

        Ok(results)
    }

    fn create_table<T: Table>(&self) -> Result<usize, duckdb::Error> {
        self.execute(&T::create_table_sql(), [])
    }

    fn insert<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error> {
        let sql = format!("INSERT INTO {} VALUES ({})", T::TABLE_NAME, T::PLACEHOLDERS);
        let params = item.to_params();
        self.execute(&sql, params.as_slice())
    }

    fn insert_batch<T: Table + ToRow>(&self, items: &[T]) -> Result<(), duckdb::Error> {
        let mut appender = self.appender(T::TABLE_NAME)?;
        for item in items {
            appender.append_row(item.to_params().as_slice())?;
        }
        appender.flush()?;
        Ok(())
    }
}

pub trait ColumnType {
    fn sql_type() -> &'static str;
}

macro_rules! impl_column_type {
    ($($ty:ty => $sql:expr),* $(,)?) => {
        $(
            impl ColumnType for $ty {
                fn sql_type() -> &'static str {
                    $sql
                }
            }
        )*
    };
}

impl_column_type! {
    i8  => "TINYINT",
    i16 => "SMALLINT",
    i32 => "INTEGER",
    i64 => "BIGINT",

    u8  => "UTINYINT",
    u16 => "USMALLINT",
    u32 => "UINTEGER",
    u64 => "UBIGINT",

    f32 => "FLOAT",
    f64 => "DOUBLE",

    bool => "BOOLEAN",

    String       => "TEXT",
    &'static str => "TEXT",
    Vec<u8>      => "BLOB",
}

impl<T: ColumnType> ColumnType for Option<T> {
    fn sql_type() -> &'static str {
        T::sql_type()
    }
}
