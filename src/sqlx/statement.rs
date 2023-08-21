use sqlx_core::{statement::Statement, Either};
use ydb_grpc_bindings::generated::ydb;

use super::prelude::*;

const NO_COLUMNS: &[YdbColumn] = &[];

#[derive(Debug, Clone)]
pub(crate) struct NamedParameters {
    names: Vec<String>,
    types: Vec<YdbTypeInfo>,
}

impl From<std::collections::HashMap<String, ydb::Type>> for NamedParameters {
    fn from(value: std::collections::HashMap<String, ydb::Type>) -> Self {
        let cap = value.len();
        let (names,types) = value.into_iter()
        .filter_map(|(k,ty)|{Some((k, YdbTypeInfo::from(&ty.r#type?)))})
        .fold((Vec::with_capacity(cap), Vec::with_capacity(cap)), |(mut names, mut types), (n,t)|{
            names.push(n);
            types.push(t);
            (names, types)
        });
        Self{names, types}
    }
}

#[derive(Debug, Clone)]
pub struct YdbStatement {
    pub (crate) query_id: String,
    pub (crate) yql: String,
    pub (crate) parameters: NamedParameters,
}

impl YdbStatement {
    pub fn query_id(&self) -> &str {
        &self.query_id
    }
}

impl Statement<'_> for YdbStatement {
    type Database = Ydb;

    fn to_owned(&self) -> YdbStatement {
        self.clone()
    }

    fn sql(&self) -> &str {
        &self.yql
    }

    fn parameters(&self) -> Option<Either<&[YdbTypeInfo], usize>> {
        Some(Either::Left(&self.parameters.types))
    }

    fn columns(&self) -> &[YdbColumn] {
        //TODO: определиться, зачем тут колонки
        NO_COLUMNS
    }

    sqlx_core::impl_statement_query!(YdbArguments);
}