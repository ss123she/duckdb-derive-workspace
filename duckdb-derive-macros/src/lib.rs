use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

struct FieldInfo<'a> {
    ident: &'a syn::Ident,
    ty: &'a syn::Type,
    column_name: String,
    is_primary_key: bool,
    foreign_key: Option<String>,
    skip: bool,
}

#[proc_macro_derive(FromRow, attributes(duckdb))]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields_info = match extract_fields(&input) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };

    let init_fields = fields_info.iter().filter(|f| !f.skip).map(|f| {
        let col_name = &f.column_name;
        quote! {
            stmt.column_index(#col_name)?
        }
    });

    let field_count = fields_info.iter().filter(|f| !f.skip).count();

    let mut db_idx = 0usize;
    let read_fields = fields_info.iter().map(|f| {
        let field_ident = f.ident;

        if f.skip {
            quote! {
                #field_ident: Default::default(),
            }
        } else {
            let current_idx = db_idx;
            db_idx += 1;
            quote! {
                #field_ident: row.get(indices[#current_idx])?,
            }
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
    let name = &input.ident;

    let fields_info = match extract_fields(&input) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };

    let active_fields: Vec<_> = fields_info.iter().filter(|f| !f.skip).collect();
    let field_count = active_fields.len();

    let placeholders = vec!["?"; field_count].join(", ");

    let field_getters = active_fields.iter().map(|f| {
        let field_ident = f.ident;
        quote! {
            &self.#field_ident as &dyn duckdb::ToSql
        }
    });

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
    let name = &input.ident;
    let table_name = name.to_string().to_lowercase();

    let fields_info = match extract_fields(&input) {
        Ok(fields) => fields,
        Err(e) => return e.to_compile_error().into(),
    };

    let column_definitions = fields_info.iter().filter(|f| !f.skip).map(|f| {
        let col_name = &f.column_name;
        let field_ty = f.ty;
        let mut constraints = String::new();

        if f.is_primary_key {
            constraints.push_str(" PRIMARY KEY");
        }

        if let Some(fk) = &f.foreign_key {
            constraints.push_str(&format!(" REFERENCES {}", fk));
        }
        quote! {
            format!(
                "{} {}{}",
                #col_name,
                <#field_ty as duckdb_derive::ColumnType>::sql_type(),
                #constraints
            )
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

fn extract_fields<'a>(input: &'a DeriveInput) -> syn::Result<Vec<FieldInfo<'a>>> {
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input.ident,
                    "Supported only on structs with named fields: `struct Foo { ... }`",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "Supported only on structs (enums and unions are not supported)",
            ));
        }
    };

    fields.iter().map(parse_field).collect()
}

fn parse_field<'a>(f: &'a syn::Field) -> syn::Result<FieldInfo<'a>> {
    let ident = f.ident.as_ref().unwrap();
    let mut column_name = ident.to_string();
    let mut is_primary_key = false;
    let mut foreign_key = None;
    let mut skip = false;

    for attr in &f.attrs {
        if attr.path().is_ident("duckdb") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("primary_key") {
                    is_primary_key = true;
                    Ok(())
                } else if meta.path.is_ident("skip") {
                    skip = true;
                    Ok(())
                } else if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    column_name = s.value();
                    Ok(())
                } else if meta.path.is_ident("foreign_key") {
                    let value = meta.value()?;
                    let s: syn::LitStr = value.parse()?;
                    foreign_key = Some(s.value());
                    Ok(())
                } else {
                    Err(meta.error(
                        "Unknown duckdb attribute. Supported: primary_key, skip, rename, foreign_key",
                    ))
                }
            })?;
        }
    }

    Ok(FieldInfo {
        ident,
        ty: &f.ty,
        column_name,
        is_primary_key,
        foreign_key,
        skip,
    })
}
