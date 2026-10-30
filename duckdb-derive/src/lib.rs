pub use duckdb_derive_macros::FromRow;

use duckdb::{Connection, Error, Params, Row};

pub trait FromRow: Sized {
    fn from_row(row: &Row<'_>) -> Result<Self, Error>;
}

pub trait DuckDbExt {
    fn query_as<'a, T: FromRow, P: Params>(&'a self, sql: &str, params: P)
    -> Result<Vec<T>, Error>;
}

impl DuckDbExt for Connection {
    fn query_as<'a, T: FromRow, P: Params>(
        &'a self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, Error> {
        let mut stmt = self.prepare(sql)?;
        let rows = stmt.query_map(params, |row| T::from_row(row))?;
        rows.collect()
    }
}
