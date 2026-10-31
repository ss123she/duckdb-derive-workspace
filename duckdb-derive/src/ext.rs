use crate::{FromRow, Table, ToRow, quote_ident, sql};

/// Extension methods for [`duckdb::Connection`].
pub trait DuckDbExt {
    /// `CREATE TABLE IF NOT EXISTS` for `T`.
    fn create_table<T: Table>(&self) -> Result<usize, duckdb::Error>;

    /// `INSERT` one row. Returns rows inserted.
    fn insert<T>(&self, item: &T) -> Result<usize, duckdb::Error>
    where
        T: Table + ToRow;

    /// `INSERT` many rows via the `Appender`. Columns are matched by position.
    fn insert_batch<T>(&self, items: &[T]) -> Result<(), duckdb::Error>
    where
        T: Table + ToRow;

    /// `INSERT ... ON CONFLICT DO UPDATE`. Returns rows affected.
    fn upsert<T>(&self, item: &T) -> Result<usize, duckdb::Error>
    where
        T: Table + ToRow;

    /// `INSERT ... ON CONFLICT DO NOTHING`. Returns 0 if the row existed.
    fn insert_or_ignore<T>(&self, item: &T) -> Result<usize, duckdb::Error>
    where
        T: Table + ToRow;

    /// `UPDATE` non-key columns by key. Returns rows updated.
    fn update<T>(&self, item: &T) -> Result<usize, duckdb::Error>
    where
        T: Table + ToRow;

    /// `DELETE` by key. Returns rows deleted.
    fn delete<T>(&self, item: &T) -> Result<usize, duckdb::Error>
    where
        T: Table + ToRow;

    /// `INSERT ... RETURNING *`, read back as `T`.
    fn insert_returning<T>(&self, item: &T) -> Result<T, duckdb::Error>
    where
        T: Table + ToRow + FromRow;

    /// Runs a query and collects the rows as `Vec<T>`.
    fn query_as<T, P>(&self, sql: &str, params: P) -> Result<Vec<T>, duckdb::Error>
    where
        T: FromRow,
        P: duckdb::Params;

    /// Runs a query and returns the first row, or `None` if there are none.
    fn query_optional<T, P>(&self, sql: &str, params: P) -> Result<Option<T>, duckdb::Error>
    where
        T: FromRow,
        P: duckdb::Params;

    /// Runs a query and returns the first row. Errors if there are none.
    fn query_one<T, P>(&self, sql: &str, params: P) -> Result<T, duckdb::Error>
    where
        T: FromRow,
        P: duckdb::Params;

    /// `SELECT *` every row of `T`'s table.
    fn select_all<T>(&self) -> Result<Vec<T>, duckdb::Error>
    where
        T: Table + FromRow;

    /// `SELECT` one row by primary key. Pass key values in field order.
    fn find<T, P>(&self, key: P) -> Result<Option<T>, duckdb::Error>
    where
        T: Table + FromRow,
        P: duckdb::Params;
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

    fn query_optional<T: FromRow, P: duckdb::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Option<T>, duckdb::Error> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.query(params)?;
        let indices = T::init_indices(rows.as_ref().expect("statement executed"))?;
        rows.next()?
            .map(|row| T::from_row(row, &indices))
            .transpose()
    }

    fn query_one<T: FromRow, P: duckdb::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<T, duckdb::Error> {
        self.query_optional(sql, params)?
            .ok_or(duckdb::Error::QueryReturnedNoRows)
    }

    fn select_all<T: Table + FromRow>(&self) -> Result<Vec<T>, duckdb::Error> {
        self.query_as(&format!("SELECT * FROM {}", quote_ident(T::TABLE_NAME)), [])
    }

    fn find<T: Table + FromRow, P: duckdb::Params>(
        &self,
        key: P,
    ) -> Result<Option<T>, duckdb::Error> {
        self.query_optional(&sql::select_by_key::<T>(), key)
    }
}
