use crate::{FromRow, Table, ToRow, quote_ident};

/// Extension methods for [`duckdb::Connection`].
pub trait DuckDbExt {
    /// Executes a SQL query and deserializes rows into `Vec<T>`.
    fn query_as<'a, T: FromRow, P: duckdb::Params>(
        &'a self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, duckdb::Error>;

    /// Creates the table for `T` if it doesn't already exist.
    fn create_table<T: Table>(&self) -> Result<usize, duckdb::Error>;

    /// Inserts a single record.
    fn insert<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error>;

    /// Inserts multiple records.
    fn insert_batch<T: Table + ToRow>(&self, items: &[T]) -> Result<(), duckdb::Error>;
}

impl DuckDbExt for duckdb::Connection {
    fn query_as<T: FromRow, P: duckdb::Params>(
        &self,
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
        let columns = T::COLUMNS
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            quote_ident(T::TABLE_NAME),
            columns,
            T::PLACEHOLDERS
        );
        self.execute(&sql, item.to_params().as_slice())
    }

    fn insert_batch<T: Table + ToRow>(&self, items: &[T]) -> Result<(), duckdb::Error> {
        let mut appender = self.appender(T::TABLE_NAME)?;
        for item in items {
            appender.append_row(item.to_params().as_slice())?;
        }
        appender.flush()
    }
}
