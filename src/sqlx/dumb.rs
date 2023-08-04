use std::{marker::PhantomData, str::FromStr};
use std::fmt::Debug;

use tonic::codegen::futures_core::future::BoxFuture;
use log::LevelFilter;
use sqlx_core::arguments::Arguments;
use sqlx_core::column::Column;
use sqlx_core::connection::{Connection, ConnectOptions};
use sqlx_core::database::Database;
use sqlx_core::row::Row;
use sqlx_core::statement::Statement;
use sqlx_core::transaction::TransactionManager;
use sqlx_core::type_info::TypeInfo;
use sqlx_core::value::{Value, ValueRef};

#[derive(Default, Debug, Clone)]
pub struct Dumb<DB> {
    _phantom: PhantomData<DB>,
}

impl<DB> std::fmt::Display for Dumb<DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<DB> PartialEq for Dumb<DB> {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl<DB: Clone> FromStr for Dumb<DB> {
    type Err = sqlx_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl<DB: 'static + Send + Clone + Debug + Sync + Database> ConnectOptions for Dumb<DB> {
    type Connection = Dumb<DB>;

    fn from_url(url: &sqlx_core::Url) -> Result<Self, sqlx_core::Error> {
        todo!()
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, sqlx_core::Error>>
    where
        Self::Connection: Sized {
        todo!()
    }

    fn log_statements(self, level: LevelFilter) -> Self {
        todo!()
    }

    fn log_slow_statements(self, level: LevelFilter, duration: std::time::Duration) -> Self {
        todo!()
    }
}

impl<DB: 'static + Send + Clone + Debug + Sync + Database> Connection for Dumb<DB> {
    type Database = DB;

    type Options = Dumb<DB>;

    fn close(self) -> BoxFuture<'static, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<sqlx_core::transaction::Transaction<'_, Self::Database>, sqlx_core::Error>>
    where
        Self: Sized {
        todo!()
    }

    fn shrink_buffers(&mut self) {
        todo!()
    }

    fn flush(&mut self) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn should_flush(&self) -> bool {
        todo!()
    }
}


impl<DB: 'static + Send + Clone + Debug + Sync + Database> TransactionManager for Dumb<DB> {
    type Database = DB;

    fn begin(
        conn: &mut <Self::Database as Database>::Connection,
    ) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn commit(
        conn: &mut <Self::Database as Database>::Connection,
    ) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn rollback(
        conn: &mut <Self::Database as Database>::Connection,
    ) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn start_rollback(conn: &mut <Self::Database as Database>::Connection) {
        todo!()
    }
}


impl<DB: 'static + Send + Clone + Debug + Sync + Database + std::marker::Unpin> Row for Dumb<DB> {
    type Database = DB;

    fn columns(&self) -> &[<Self::Database as Database>::Column] {
        todo!()
    }

    fn try_get_raw<I>(
        &self,
        index: I,
    ) -> Result<<Self::Database as sqlx_core::database::HasValueRef<'_>>::ValueRef, sqlx_core::Error>
    where
        I: sqlx_core::column::ColumnIndex<Self> {
        todo!()
    }
}

impl<DB: 'static + Send + Clone + Debug + Sync + Database> Extend<Self> for Dumb<DB> {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        todo!()
    }
}

impl<DB: 'static + Send + Clone + Debug + Sync + Database> Column for Dumb<DB> {
    type Database = DB;

    fn ordinal(&self) -> usize {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }

    fn type_info(&self) -> &<Self::Database as Database>::TypeInfo {
        todo!()
    }
}

impl<DB: 'static + Send + Clone + Debug + Sync + Database> TypeInfo for Dumb<DB> {
    fn is_null(&self) -> bool {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }
}

impl<DB: 'static + Send + Clone + Debug + Sync + Database> Value for Dumb<DB> {
    type Database = DB;

    fn as_ref(&self) -> <Self::Database as sqlx_core::database::HasValueRef<'_>>::ValueRef {
        todo!()
    }

    fn type_info(&self) -> std::borrow::Cow<'_, <Self::Database as Database>::TypeInfo> {
        todo!()
    }

    fn is_null(&self) -> bool {
        todo!()
    }
}


impl<'a, DB: 'static + Send + Clone + Debug + Sync + Database + Default> Arguments<'a> for Dumb<DB> {
    type Database = DB;

    fn reserve(&mut self, additional: usize, size: usize) {
        todo!()
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'a + Send + sqlx_core::encode::Encode<'a, Self::Database> + sqlx_core::types::Type<Self::Database> {
        todo!()
    }
}

impl<'a, DB: 'static + Send + Clone + Debug + Sync + Database> Statement<'a> for Dumb<DB> {
    type Database = DB;

    fn to_owned(&self) -> <Self::Database as sqlx_core::database::HasStatement<'static>>::Statement {
        todo!()
    }

    fn sql(&self) -> &str {
        todo!()
    }

    fn parameters(&self) -> Option<sqlx_core::Either<&[<Self::Database as Database>::TypeInfo], usize>> {
        todo!()
    }

    fn columns(&self) -> &[<Self::Database as Database>::Column] {
        todo!()
    }

    fn query(&self) -> sqlx_core::query::Query<'_, Self::Database, <Self::Database as sqlx_core::database::HasArguments<'_>>::Arguments> {
        todo!()
    }

    fn query_with<'s, A>(&'s self, arguments: A) -> sqlx_core::query::Query<'s, Self::Database, A>
    where
        A: sqlx_core::arguments::IntoArguments<'s, Self::Database> {
        todo!()
    }

    fn query_as<O>(
        &self,
    ) -> sqlx_core::query_as::QueryAs<'_, Self::Database, O, <Self::Database as sqlx_core::database::HasArguments<'_>>::Arguments>
    where
        O: for<'r> sqlx_core::from_row::FromRow<'r, <Self::Database as Database>::Row> {
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
    ) -> sqlx_core::query_scalar::QueryScalar<'_, Self::Database, O, <Self::Database as sqlx_core::database::HasArguments<'_>>::Arguments>
    where
        (O,): for<'r> sqlx_core::from_row::FromRow<'r, <Self::Database as Database>::Row> {
        todo!()
    }

    fn query_scalar_with<'s, O, A>(&'s self, arguments: A) -> sqlx_core::query_scalar::QueryScalar<'s, Self::Database, O, A>
    where
        (O,): for<'r> sqlx_core::from_row::FromRow<'r, <Self::Database as Database>::Row>,
        A: sqlx_core::arguments::IntoArguments<'s, Self::Database> {
        todo!()
    }
}

impl<'a, DB: 'static + Send + Clone + Debug + Sync + Database> ValueRef<'a> for Dumb<DB> {
    type Database = DB;

    fn to_owned(&self) -> <Self::Database as Database>::Value {
        todo!()
    }

    fn type_info(&self) -> std::borrow::Cow<'_, <Self::Database as Database>::TypeInfo> {
        todo!()
    }

    fn is_null(&self) -> bool {
        todo!()
    }
}