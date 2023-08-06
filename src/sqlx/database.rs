use std::fmt::Display;

use sqlx_core::arguments::IntoArguments;
use sqlx_core::{row::Row, column::Column, type_info::TypeInfo, statement::Statement, arguments::Arguments};
use sqlx_core::Either;
use sqlx_core::database::{Database, HasArguments, HasStatement, HasValueRef};
use ydb_grpc_bindings::generated::ydb;


use super::{YdbValueRef, YdbTypeInfo, YdbValue};
use super::{YdbRow, YdbQueryResult, YdbColumn};
use super::{dumb::Dumb, YdbConnection};

pub type YdbArgumentBuffer = sqlx_core::HashMap<String, ydb::TypedValue>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Ydb;

impl Database for Ydb {
    type Connection = YdbConnection;

    type TransactionManager = Dumb<Self>;

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

#[derive(Debug, Default)]
pub struct YdbArguments;

impl<'q> Arguments<'q> for YdbArguments {
    type Database = Ydb;

    fn reserve(&mut self, additional: usize, size: usize) {
        todo!()
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Send + sqlx_core::encode::Encode<'q, Self::Database> + sqlx_core::types::Type<Self::Database> {
        todo!()
    }
}

impl<'a> IntoArguments<'a, Ydb> for YdbArguments {
    fn into_arguments(self) -> YdbArguments {
        todo!()
    }
}

impl <'a> HasStatement<'a> for Ydb {
    type Database = Self;

    type Statement = YdbStatement;
}

pub struct YdbStatement;

impl Statement<'_> for YdbStatement {
    type Database = Ydb;

    fn to_owned(&self) -> YdbStatement {
        todo!()
    }

    fn sql(&self) -> &str {
        todo!()
    }

    fn parameters(&self) -> Option<Either<&[YdbTypeInfo], usize>> {
        todo!()
    }

    fn columns(&self) -> &[YdbColumn] {
        todo!()
    }

    fn query(&self) -> sqlx_core::query::Query<'_, Ydb, YdbArguments> {
        todo!()
    }

    fn query_with<'s, A>(&'s self, arguments: A) -> sqlx_core::query::Query<'s, Ydb, A>
    where
        A: sqlx_core::arguments::IntoArguments<'s, Self::Database> {
        todo!()
    }

    fn query_as<O>(
        &self,
    ) -> sqlx_core::query_as::QueryAs<'_, Ydb, O, YdbArguments>
    where
        O: for<'r> sqlx_core::from_row::FromRow<'r, YdbRow> {
        todo!()
    }

    fn query_as_with<'s, O, A>(&'s self, arguments: A) -> sqlx_core::query_as::QueryAs<'s, Self::Database, O, A>
    where
        O: for<'r> sqlx_core::from_row::FromRow<'r, <Self::Database as Database>::Row>,
        A: sqlx_core::arguments::IntoArguments<'s, Self::Database> {
        todo!()
    }

    fn query_scalar<O>(
        &self,
    ) -> sqlx_core::query_scalar::QueryScalar<'_, Ydb, O, YdbArguments>
    where
        (O,): for<'r> sqlx_core::from_row::FromRow<'r, YdbRow> {
        todo!()
    }

    fn query_scalar_with<'s, O, A>(&'s self, arguments: A) -> sqlx_core::query_scalar::QueryScalar<'s, Ydb, O, A>
    where
        (O,): for<'r> sqlx_core::from_row::FromRow<'r, YdbRow>,
        A: sqlx_core::arguments::IntoArguments<'s, Ydb> {
        todo!()
    }
}

impl <'a> HasValueRef<'a> for Ydb {
    type Database = Self;
    type ValueRef = YdbValueRef<'a>;
}
