#![forbid(unsafe_code)]

mod attrs;
mod row;
mod table;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derives `FromRow` for deserializing query rows by column names.
#[proc_macro_derive(FromRow, attributes(duckdb))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    row::expand_from_row(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives `ToRow` to convert struct fields into SQL parameters.
#[proc_macro_derive(ToRow, attributes(duckdb))]
pub fn derive_to_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    row::expand_to_row(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives `Table`, which generates `CREATE TABLE IF NOT EXISTS` DDL and exposes the table name and primary key.
///
/// Non-`Option` fields get `NOT NULL`.
///
/// # Struct attributes
/// - `table = "name"`: override the table name (default: lowercased struct name).
///
/// # Field attributes
/// - `primary_key`: mark as `PRIMARY KEY`. Several fields make a composite
///   key (one table-level constraint, in field order).
/// - `foreign_key = "tbl(col)"`: add `REFERENCES tbl(col)`.
/// - `unique`: add `UNIQUE`.
/// - `default = "SQL"`: add `DEFAULT SQL`. The value is raw SQL, so string defaults need inner quotes: `default = "'new'"`.
/// - `check = "SQL"`: add `CHECK (SQL)`.
/// - `sql_type = "TYPE"`: override the inferred SQL type.
/// - `extra = "SQL"`: append any other raw SQL.
/// - `rename = "name"`: custom column name.
/// - `skip`: exclude the field from the schema.
///
/// # Example
/// ```ignore
/// #[derive(Table)]
/// #[duckdb(table = "ducks")]
/// struct Duck {
///     #[duckdb(primary_key)]
///     id: i32,
///     #[duckdb(unique)]
///     name: String,
///     #[duckdb(default = "0", check = "age >= 0")]
///     age: u8,
///     bio: Option<String>,
/// }
/// ```

#[proc_macro_derive(Table, attributes(duckdb))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    table::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
