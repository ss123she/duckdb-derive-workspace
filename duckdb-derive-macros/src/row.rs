use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::attrs::extract_fields;

pub fn expand_to_row(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = extract_fields(input)?;
    let active: Vec<_> = fields.iter().filter(|f| !f.skip).collect();

    let count = active.len();
    let placeholders = vec!["?"; count].join(", ");
    let columns = active.iter().map(|f| &f.column_name);
    let getters = active.iter().map(|f| {
        let ident = f.ident;
        quote! { &self.#ident as &dyn duckdb::ToSql }
    });

    Ok(quote! {
        impl #impl_generics duckdb_derive::ToRow for #name #ty_generics #where_clause {
            const COLUMN_COUNT: usize = #count;
            const COLUMNS: &'static [&'static str] = &[ #(#columns),* ];
            const PLACEHOLDERS: &'static str = #placeholders;

            fn to_params(&self) -> Vec<&dyn duckdb::ToSql> {
                vec![ #(#getters),* ]
            }
        }
    })
}

pub fn expand_from_row(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = extract_fields(input)?;

    let active_count = fields.iter().filter(|f| !f.skip).count();
    let col_lookups = fields.iter().filter(|f| !f.skip).map(|f| {
        let col = &f.column_name;
        quote! { stmt.column_index(#col)? }
    });

    let mut db_idx = 0usize;
    let read_fields = fields.iter().map(|f| {
        let ident = f.ident;
        if f.skip {
            quote! { #ident: Default::default(), }
        } else {
            let i = db_idx;
            db_idx += 1;
            quote! { #ident: row.get(indices[#i])?, }
        }
    });

    Ok(quote! {
        impl #impl_generics duckdb_derive::FromRow for #name #ty_generics #where_clause {
            type Indices = [usize; #active_count];

            fn init_indices(stmt: &duckdb::Statement<'_>) -> Result<Self::Indices, duckdb::Error> {
                Ok([ #(#col_lookups),* ])
            }

            fn from_row(row: &duckdb::Row<'_>, indices: &Self::Indices) -> Result<Self, duckdb::Error> {
                Ok(Self { #(#read_fields)* })
            }
        }
    })
}
