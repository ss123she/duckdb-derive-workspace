pub use duckdb_derive_macros::FromRow;

pub trait FromRow: Sized {
    type Indices;
    fn init_indices(stmt: &duckdb::Statement<'_>) -> Result<Self::Indices, duckdb::Error>;
    fn from_row(row: &duckdb::Row<'_>, indices: &Self::Indices) -> Result<Self, duckdb::Error>;
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
}

pub trait ColumnType {
    fn sql_type() -> &'static str;
}

impl ColumnType for i32 {
    fn sql_type() -> &'static str {
        "INTEGER"
    }
}
impl ColumnType for String {
    fn sql_type() -> &'static str {
        "TEXT"
    }
}
impl ColumnType for f64 {
    fn sql_type() -> &'static str {
        "DOUBLE"
    }
}
impl ColumnType for bool {
    fn sql_type() -> &'static str {
        "BOOLEAN"
    }
}
