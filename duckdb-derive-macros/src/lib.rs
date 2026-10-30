use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

fn data_fields(data: Data) -> syn::punctuated::Punctuated<syn::Field, syn::token::Comma> {
    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("FromRow can only be derived on structs with named fields"),
        },
        _ => panic!("FromRow can only be derived on structs"),
    }
}

#[proc_macro_derive(FromRow, attributes(duckdb))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = data_fields(input.data);

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

#[proc_macro_derive(ToRow, attributes(duckdb))]
pub fn derive_to_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = data_fields(input.data);
    let field_count = fields.len();

    let field_getters = fields.iter().map(|f| {
        let field_ident = &f.ident;
        quote! {
            &self.#field_ident as &dyn duckdb::ToSql
        }
    });

    let placeholders = vec!["?"; field_count].join(", ");

    let name = input.ident;

    let expanded = quote! {
        impl duckdb_derive::ToRow for #name {
            const COLUMN_COUNT: usize = #field_count;
            const PLACEHOLDERS: &'static str = #placeholders;

            fn to_params(&self) -> Vec<&dyn duckdb::ToSql> {
                vec![
                    #(#field_getters),*
                ]
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

    let fields = data_fields(input.data);

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
