use crate::{Table, ToRow, quote_ident};

fn require_pk<T: Table>() {
    const {
        assert!(
            !T::PRIMARY_KEY.is_empty(),
            "this operation needs at least one #[duckdb(primary_key)] field"
        )
    };
}

fn is_pk<T: Table>(col: &str) -> bool {
    T::PRIMARY_KEY.iter().any(|p| *p == col)
}

fn pk_predicate<T: Table>() -> String {
    T::PRIMARY_KEY
        .iter()
        .map(|c| format!("{} = ?", quote_ident(c)))
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub(crate) fn insert<T: Table + ToRow>(suffix: &str) -> String {
    let columns = T::COLUMNS
        .iter()
        .map(|c| quote_ident(c))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "INSERT INTO {} ({}) VALUES ({}){}",
        quote_ident(T::TABLE_NAME),
        columns,
        T::PLACEHOLDERS,
        suffix
    )
}

pub(crate) fn upsert<T: Table + ToRow>() -> String {
    require_pk::<T>();
    let conflict = T::PRIMARY_KEY
        .iter()
        .map(|c| quote_ident(c))
        .collect::<Vec<_>>()
        .join(", ");
    let sets: Vec<String> = T::COLUMNS
        .iter()
        .copied()
        .filter(|c| !is_pk::<T>(c))
        .map(|c| format!("{q} = excluded.{q}", q = quote_ident(c)))
        .collect();
    let action = if sets.is_empty() {
        "DO NOTHING".to_string()
    } else {
        format!("DO UPDATE SET {}", sets.join(", "))
    };
    insert::<T>(&format!(" ON CONFLICT ({conflict}) {action}"))
}

pub(crate) fn update<T: Table + ToRow>() -> Option<String> {
    require_pk::<T>();
    let sets: Vec<String> = T::COLUMNS
        .iter()
        .copied()
        .filter(|c| !is_pk::<T>(c))
        .map(|c| format!("{} = ?", quote_ident(c)))
        .collect();
    if sets.is_empty() {
        return None;
    }
    Some(format!(
        "UPDATE {} SET {} WHERE {}",
        quote_ident(T::TABLE_NAME),
        sets.join(", "),
        pk_predicate::<T>()
    ))
}

pub(crate) fn delete<T: Table>() -> String {
    require_pk::<T>();
    format!(
        "DELETE FROM {} WHERE {}",
        quote_ident(T::TABLE_NAME),
        pk_predicate::<T>()
    )
}

pub(crate) fn split_params<T: Table + ToRow>(
    item: &T,
) -> (Vec<&dyn duckdb::ToSql>, Vec<&dyn duckdb::ToSql>) {
    let mut rest = Vec::new();
    let mut key = Vec::new();
    for (col, param) in T::COLUMNS.iter().zip(item.to_params()) {
        if is_pk::<T>(col) {
            key.push(param)
        } else {
            rest.push(param)
        }
    }
    (rest, key)
}

pub(crate) fn select_by_key<T: Table>() -> String {
    require_pk::<T>();
    format!(
        "SELECT * FROM {} WHERE {}",
        quote_ident(T::TABLE_NAME),
        pk_predicate::<T>(),
    )
}
