/// Maps a Rust type to its DuckDB SQL representation and nullability constraint.
pub trait ColumnType {
    /// DuckDB SQL type string (e.g. `"INTEGER"`, `"TEXT"`).
    fn sql_type() -> &'static str;

    /// Whether this type is nullable. Defaults to `false` (only `Option<T>` is `true`).
    fn is_nullable() -> bool {
        false
    }
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

#[cfg(feature = "chrono")]
impl_column_type! {
    chrono::NaiveDate             => "DATE",
    chrono::NaiveTime             => "TIME",
    chrono::NaiveDateTime         => "TIMESTAMP",
    chrono::DateTime<chrono::Utc> => "TIMESTAMPTZ",
}

#[cfg(feature = "uuid")]
impl_column_type! {
    uuid::Uuid => "UUID",
}

impl<T: ColumnType> ColumnType for Option<T> {
    fn sql_type() -> &'static str {
        T::sql_type()
    }

    fn is_nullable() -> bool {
        true
    }
}

#[cfg(feature = "rust_decimal")]
impl ColumnType for rust_decimal::Decimal {
    fn sql_type() -> &'static str {
        "DECIMAL"
    }
}

#[cfg(feature = "serde_json")]
impl ColumnType for serde_json::Value {
    fn sql_type() -> &'static str {
        "JSON"
    }
}
