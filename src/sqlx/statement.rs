use sqlx_core::{statement::Statement, Either, query::Query};

use super::{Ydb, YdbTypeInfo, YdbColumn, YdbArguments, YdbRow};

#[derive(Debug, Clone)]
pub struct YdbStatement {
    yql: String,
    parameters: Vec<YdbTypeInfo>,
    columns: Vec<YdbColumn>,
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
        Some(Either::Left(&self.parameters))
    }

    fn columns(&self) -> &[YdbColumn] {
        &self.columns
    }

    fn query(&self) -> Query<'_, Ydb, YdbArguments> {
        todo!()
    }

    fn query_with<'s, A>(&'s self, arguments: A) -> sqlx_core::query::Query<'s, Ydb, A>
    where
        A: sqlx_core::arguments::IntoArguments<'s, Ydb> {
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
        O: for<'r> sqlx_core::from_row::FromRow<'r, YdbRow>,
        A: sqlx_core::arguments::IntoArguments<'s, Ydb> {
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