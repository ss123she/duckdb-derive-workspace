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
impl ColumnType for i64 {
    fn sql_type() -> &'static str {
        "BIGINT"
    }
}
impl ColumnType for i16 {
    fn sql_type() -> &'static str {
        "SMALLINT"
    }
}
impl ColumnType for i8 {
    fn sql_type() -> &'static str {
        "TINYINT"
    }
}
impl ColumnType for u32 {
    fn sql_type() -> &'static str {
        "UINTEGER"
    }
}
impl ColumnType for u64 {
    fn sql_type() -> &'static str {
        "UBIGINT"
    }
}
impl ColumnType for f32 {
    fn sql_type() -> &'static str {
        "FLOAT"
    }
}
impl ColumnType for Vec<u8> {
    fn sql_type() -> &'static str {
        "BLOB"
    }
}
impl ColumnType for &'static str {
    fn sql_type() -> &'static str {
        "TEXT"
    }
}
