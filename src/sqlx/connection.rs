use std::str::FromStr;
use super::database::Ydb;
use sqlx_core::transaction::Transaction;
use sqlx_core::pool::MaybePoolConnection;
use tonic::codegen::futures_core::future::BoxFuture;
use sqlx_core::connection::{ConnectOptions as XConnectOptions, Connection};
use crate::AsciiValue;
use crate::auth::UpdatableToken;
use crate::client::YdbEndpoint;
use super::YdbConnection;


#[derive(Debug, Clone)]
pub struct ConnectOptions {
    endpoint: YdbEndpoint,
    db_name: AsciiValue,
    creds: UpdatableToken,
}


impl FromStr for ConnectOptions {
    type Err = sqlx_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl XConnectOptions for ConnectOptions {
    type Connection = YdbConnection;

    fn from_url(url: &sqlx_core::Url) -> Result<Self, sqlx_core::Error> {
        todo!()
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, sqlx_core::Error>>
    where
        Self::Connection: Sized {
        let channel = self.endpoint.make_endpoint().connect_lazy();
        let conn = crate::YdbConnection::new(channel, self.db_name.clone(), self.creds.clone());
        Box::pin(async move{
            Ok(conn)
        })
    }

    fn log_statements(self, level: log::LevelFilter) -> Self {
        todo!()
    }

    fn log_slow_statements(self, level: log::LevelFilter, duration: std::time::Duration) -> Self {
        todo!()
    }
}

impl Connection for YdbConnection {
    type Database = Ydb;

    type Options = ConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), sqlx_core::Error>> {
        Box::pin(async move{
            self.close_session().await?;
            Ok(())
        })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), sqlx_core::Error>> {
        self.close_session_hard();
        Box::pin(async {Ok(())})
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Ydb>, sqlx_core::Error>> where Self: Sized {
        Transaction::begin(MaybePoolConnection::Connection(self))
    }
    fn shrink_buffers(&mut self) {}
    fn flush(&mut self) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        Box::pin(futures::future::ok(()))
    }
    fn should_flush(&self) -> bool {false}
}
