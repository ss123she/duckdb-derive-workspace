use crate::{FromRow, Table, ToRow, sql};

/// Extension methods for [`duckdb::Connection`].
pub trait DuckDbExt {
    /// Executes a SQL query and deserializes rows into `Vec<T>`.
    fn query_as<'a, T: FromRow, P: duckdb::Params>(
        &'a self,
        sql: &str,
        params: P,
    ) -> Result<Vec<T>, duckdb::Error>;

    /// `CREATE TABLE IF NOT EXISTS` for `T`.
    fn create_table<T: Table>(&self) -> Result<usize, duckdb::Error>;

    /// `INSERT` one row; returns rows inserted.
    fn insert<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error>;

    /// `INSERT` many rows via the `Appender`; columns are matched by position.
    fn insert_batch<T: Table + ToRow>(&self, items: &[T]) -> Result<(), duckdb::Error>;

    /// `INSERT ... ON CONFLICT DO UPDATE`; returns rows affected.
    fn upsert<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error>;

    /// `INSERT ... ON CONFLICT DO NOTHING`; returns 0 if the row existed.
    fn insert_or_ignore<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error>;

    /// `UPDATE` non-key columns by key; returns rows updated.
    fn update<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error>;

    /// `DELETE` by key; returns rows deleted.
    fn delete<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error>;

    /// `INSERT ... RETURNING *`, read back as `T`.
    fn insert_returning<T: Table + ToRow + FromRow>(&self, item: &T) -> Result<T, duckdb::Error>;
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
        self.execute(&sql::insert::<T>(""), item.to_params().as_slice())
    }

    fn insert_batch<T: Table + ToRow>(&self, items: &[T]) -> Result<(), duckdb::Error> {
        let mut appender = self.appender(T::TABLE_NAME)?;
        for item in items {
            appender.append_row(item.to_params().as_slice())?;
        }
        appender.flush()
    }

    fn insert_or_ignore<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error> {
        self.execute(
            &sql::insert::<T>(" ON CONFLICT DO NOTHING"),
            item.to_params().as_slice(),
        )
    }

    fn upsert<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error> {
        self.execute(&sql::upsert::<T>(), item.to_params().as_slice())
    }

    fn update<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error> {
        let Some(sql) = sql::update::<T>() else {
            return Ok(0);
        };
        let (mut params, key) = sql::split_params(item);
        params.extend(key);
        self.execute(&sql, params.as_slice())
    }

    fn delete<T: Table + ToRow>(&self, item: &T) -> Result<usize, duckdb::Error> {
        let (_, key) = sql::split_params(item);
        self.execute(&sql::delete::<T>(), key.as_slice())
    }

    fn insert_returning<T: Table + ToRow + FromRow>(&self, item: &T) -> Result<T, duckdb::Error> {
        let sql = sql::insert::<T>(" RETURNING *");
        let params = item.to_params();
        let mut stmt = self.prepare(&sql)?;
        let mut rows = stmt.query(params.as_slice())?;
        let indices = T::init_indices(rows.as_ref().expect("statement executed"))?;
        match rows.next()? {
            Some(row) => T::from_row(row, &indices),
            None => Err(duckdb::Error::QueryReturnedNoRows),
        }
    }
}
