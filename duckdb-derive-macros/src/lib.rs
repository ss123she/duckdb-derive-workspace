use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FromRow, attributes(duckdb))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("FromRow can only be derived on structs with named fields"),
        },
        _ => panic!("FromRow can only be derived on structs"),
    };

    let field_count = fields.len();

    let init_fields = fields.iter().map(|f| {
        let column_name = f.ident.as_ref().unwrap().to_string();
        quote! {
            stmt.column_index(#column_name)?
        }
    });

    let read_fields = fields.iter().enumerate().map(|(idx, f)| {
        let field_name = &f.ident;
        quote! {
            #field_name: row.get(indices[#idx])?,
        }
    });

    let expanded = quote! {
        impl duckdb_derive::FromRow for #name {
            type Indices = [usize; #field_count];

            fn init_indices(stmt: &duckdb::Statement<'_>) -> Result<Self::Indices, duckdb::Error> {
                Ok([
                    #(#init_fields),*
                ])
            }

            fn from_row(row: &duckdb::Row<'_>, indices: &Self::Indices) -> Result<Self, duckdb::Error> {
                Ok(Self {
                    #(#read_fields)*
                })
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Table, attributes(duckdb))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let table_name = name.to_string().to_lowercase();

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Table can only be derived on structs with named fields"),
        },
        _ => panic!("Table can only be derived on structs"),
    };

    let column_definitions = fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap().to_string();
        let field_ty = &f.ty;

        quote! {
            format!("{} {}", #field_name, <#field_ty as duckdb_derive::ColumnType>::sql_type())
        }
    });

    let expanded = quote! {
        impl duckdb_derive::Table for #name {
            const TABLE_NAME: &'static str = #table_name;

            fn create_table_sql() -> String {
                let cols = vec![
                    #(#column_definitions),*
                ];

                format!(
                    "CREATE TABLE IF NOT EXISTS {} ({});",
                    Self::TABLE_NAME,
                    cols.join(", ")
                )
            }
        }
    };

    expanded.into()
}
