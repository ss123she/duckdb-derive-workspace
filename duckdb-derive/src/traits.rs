/// Deserializes a DuckDB row into a Rust struct.
///
/// derived via `#[derive(FromRow)]`.
pub trait FromRow: Sized {
    /// Cached array of column positions.
    type Indices;

    /// Resolves column names to indices from statement metadata.
    fn init_indices(stmt: &duckdb::Statement<'_>) -> Result<Self::Indices, duckdb::Error>;

    /// Constructs an instance of `Self` from a row using indices.
    fn from_row(row: &duckdb::Row<'_>, indices: &Self::Indices) -> Result<Self, duckdb::Error>;
}

/// Converts a Rust structs into SQL params for queries and inserts.
///
/// derived via `#[derive(ToRow)]`.
pub trait ToRow {
    /// Number of columns.
    const COLUMN_COUNT: usize;

    /// Column names, in field order.
    const COLUMNS: &'static [&'static str];

    /// comma-separated placeholder str.
    const PLACEHOLDERS: &'static str;

    /// Returns a list of preferences to fields implementing [`duckdb::ToSql`].
    fn to_params(&self) -> Vec<&dyn duckdb::ToSql>;
}

/// Provides table metadata and sql schema for DuckDB.
///
/// derived via `#[derive(Table)]`.
pub trait Table {
    /// Target table name in DuckDB
    const TABLE_NAME: &'static str;

    /// Primary key column names, in field order. Empty if none declared.
    const PRIMARY_KEY: &'static [&'static str] = &[];

    /// Generates the `CREATE TABLE IF NOT EXISTS` statement based on struct fields.
    fn create_table_sql() -> String;
}
