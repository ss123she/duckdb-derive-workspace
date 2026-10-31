use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attrs::extract_fields;

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn parse_table_name(input: &DeriveInput) -> syn::Result<String> {
    let mut table_name = input.ident.to_string().to_lowercase();
    for attr in &input.attrs {
        if attr.path().is_ident("duckdb") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table") {
                    table_name = meta.value()?.parse::<syn::LitStr>()?.value();
                    Ok(())
                } else {
                    Err(meta.error("Supported struct attributes: table"))
                }
            })?;
        }
    }
    Ok(table_name)
}

pub fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let table_name = parse_table_name(input)?;
    let fields = extract_fields(input)?;

    let column_defs = fields.iter().filter(|f| !f.skip).map(|f| {
        let quoted_col = quote_ident(&f.column_name);
        let ty = f.ty;
        let is_pk = f.is_primary_key;

        let mut constraints = Vec::new();
        if is_pk {
            constraints.push("PRIMARY KEY".to_string());
        }
        if let Some(fk) = &f.foreign_key {
            constraints.push(format!("REFERENCES {fk}"));
        }
        if let Some(extra) = &f.extra {
            constraints.push(extra.clone());
        }
        let constraints_str = if constraints.is_empty() {
            String::new()
        } else {
            format!(" {}", constraints.join(" "))
        };

        let extra_has_not_null = f
            .extra
            .as_deref()
            .is_some_and(|e| e.to_uppercase().contains("NOT NULL"));

        let col_type = match &f.sql_type_override {
            Some(custom) => quote! { #custom },
            None => quote! { <#ty as duckdb_derive::ColumnType>::sql_type() },
        };

        quote! {
            {
                let not_null = if !<#ty as duckdb_derive::ColumnType>::is_nullable()
                    && !#is_pk
                    && !#extra_has_not_null
                {
                    " NOT NULL"
                } else {
                    ""
                };
                format!("{} {}{}{}", #quoted_col, #col_type, not_null, #constraints_str)
            }
        }
    });

    Ok(quote! {
        impl #impl_generics duckdb_derive::Table for #name #ty_generics #where_clause {
            const TABLE_NAME: &'static str = #table_name;

            fn create_table_sql() -> String {
                let cols = [ #(#column_defs),* ];
                format!(
                    "CREATE TABLE IF NOT EXISTS {} ({});",
                    duckdb_derive::quote_ident(Self::TABLE_NAME),
                    cols.join(", ")
                )
            }
        }
    })
}
