use syn::{Data, DeriveInput, Fields};

pub struct FieldInfo<'a> {
    pub ident: &'a syn::Ident,
    pub ty: &'a syn::Type,
    pub column_name: String,
    pub is_primary_key: bool,
    pub foreign_key: Option<String>,
    pub skip: bool,
    pub sql_type_override: Option<String>,
    pub extra: Option<String>,
}

pub fn extract_fields(input: &DeriveInput) -> syn::Result<Vec<FieldInfo<'_>>> {
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
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

fn parse_field(f: &syn::Field) -> syn::Result<FieldInfo<'_>> {
    let ident = f.ident.as_ref().unwrap();
    let mut info = FieldInfo {
        ident,
        ty: &f.ty,
        column_name: ident.to_string(),
        is_primary_key: false,
        foreign_key: None,
        skip: false,
        sql_type_override: None,
        extra: None,
    };

    for attr in &f.attrs {
        if !attr.path().is_ident("duckdb") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            let path = &meta.path;
            if path.is_ident("primary_key") {
                info.is_primary_key = true;
            } else if path.is_ident("skip") {
                info.skip = true;
            } else if path.is_ident("rename") {
                info.column_name = meta.value()?.parse::<syn::LitStr>()?.value();
            } else if path.is_ident("sql_type") {
                info.sql_type_override = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            } else if path.is_ident("extra") {
                info.extra = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            } else if path.is_ident("foreign_key") {
                info.foreign_key = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            } else {
                return Err(meta.error(
                    "Unknown duckdb attribute. Supported: primary_key, skip, rename, foreign_key, sql_type, extra",
                ));
            }
            Ok(())
        })?;
    }

    Ok(info)
}
