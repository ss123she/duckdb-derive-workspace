pub use duckdb_derive_macros::FromRow;

pub trait FromRow: Sized {
    type Indices;
    fn init_indices(stmt: &duckdb::Statement<'_>) -> Result<Self::Indices, duckdb::Error>;
    fn from_row(row: &duckdb::Row<'_>, indices: &Self::Indices) -> Result<Self, duckdb::Error>;
}

pub trait DuckDbExt {
    fn query_as<'a, T: FromRow, P: duckdb::Params>(
        &'a self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, duckdb::Error>;
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
}
