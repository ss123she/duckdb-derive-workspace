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

/// Derives `Table` to generate `CREATE TABLE IF NOT EXISTS` DDL.
///
/// Non-`Option` fields default to `NOT NULL`.
///
/// # Struct Attributes (`#[duckdb(...)]`)
/// - `table = "name"`: override table name (default: lowercase struct name).
///
/// # Field Attributes (`#[duckdb(...)]`)
/// - `primary_key`: mark as `PRIMARY KEY` (suppresses duplicate `NOT NULL`).
/// - `foreign_key = "tbl(col)"`: add `REFERENCES tbl(col)`.
/// - `sql_type = "TYPE"`: override inferred SQL type.
/// - `extra = "SQL"`: append raw SQL (e.g. `DEFAULT 0`, `CHECK (...)`).
/// - `rename = "name"`: custom SQL column name.
/// - `skip`: exclude field from schema.
#[proc_macro_derive(Table, attributes(duckdb))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    table::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
