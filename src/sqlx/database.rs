use sqlx_core::arguments::IntoArguments;
use sqlx_core::arguments::Arguments;
use sqlx_core::database::{Database, HasArguments, HasStatement, HasValueRef};
use ydb_grpc_bindings::generated::ydb;

use super::prelude::*;

pub type YdbArgumentBuffer = std::collections::HashMap<String, ydb::TypedValue>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Ydb;

impl Database for Ydb {
    type Connection = YdbConnection;

    type TransactionManager = YdbTransactionManager;

    type Row = YdbRow;

    type QueryResult = YdbQueryResult;

    type Column = YdbColumn;

    type TypeInfo = YdbTypeInfo;

    type Value = YdbValue;

    const NAME: &'static str = "Ydb";

    const URL_SCHEMES: &'static [&'static str] = &["ydb", "ydbs"];
}

impl<'a> HasArguments<'a> for Ydb {
    type Database = Self;

    type Arguments = YdbArguments;

    type ArgumentBuffer=YdbArgumentBuffer;
}

#[derive(Debug, Default, Clone)]
pub struct YdbArguments(pub(crate) YdbArgumentBuffer);

impl<'q> Arguments<'q> for YdbArguments {
    type Database = Ydb;

    fn reserve(&mut self, _additional: usize, _size: usize) {
        //TODO: implement me
    }

    fn add<T>(&mut self, value: T)
    where T: 'q + Send + sqlx_core::encode::Encode<'q, Ydb> + sqlx_core::types::Type<Ydb> {
        let _ = value.encode(&mut self.0);
    }
}

impl<'a> IntoArguments<'a, Ydb> for &YdbArguments {
    fn into_arguments(self) -> YdbArguments {
        self.clone()
    }
}

impl IntoArguments<'_, Ydb> for YdbArguments {
    fn into_arguments(self) -> YdbArguments {
        self
    }
}

impl <'a> HasStatement<'a> for Ydb {
    type Database = Self;

    type Statement = YdbStatement;
}

impl <'a> HasValueRef<'a> for Ydb {
    type Database = Self;
    type ValueRef = YdbValueRef<'a>;
}
