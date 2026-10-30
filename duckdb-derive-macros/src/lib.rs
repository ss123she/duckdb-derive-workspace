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

    let getters = fields.iter().map(|f| {
        let field_name = &f.ident;
        let column_name = field_name.as_ref().unwrap().to_string();

        quote! {
            #field_name: row.get(#column_name)?,
        }
    });

    let expanded = quote! {
        impl duckdb_derive::FromRow for #name {
            fn from_row(row: &duckdb::Row<'_>) -> Result<Self, duckdb::Error> {
                Ok(Self {
                    #(#getters)*
                })
            }
        }
    };

    expanded.into()
}
