use std::str::FromStr;
use super::YdbError;
use super::database::Ydb;
use sqlx_core::transaction::{Transaction, TransactionManager};
use sqlx_core::pool::MaybePoolConnection;
use tonic::codegen::futures_core::future::BoxFuture;
use sqlx_core::connection::{ConnectOptions, Connection};
use ydb_grpc_bindings::generated::ydb;
use ydb::table::{TransactionControl, BeginTransactionRequest};
use ydb::table::transaction_control::TxSelector;
use ydb::table::TransactionSettings;
use ydb::table::transaction_settings::TxMode;
use crate::{AsciiValue, YdbTransaction};
use crate::auth::UpdatableToken;
use crate::client::YdbEndpoint;

use crate::payload::YdbResponseWithResult;

pub struct YdbConnection {
    inner: crate::YdbConnection<UpdatableToken>,
    options: YdbConnectOptions,
    tx_control: TransactionControl,
}

#[derive(Debug, Clone)]
pub struct YdbConnectOptions {
    endpoint: YdbEndpoint,
    db_name: AsciiValue,
    creds: UpdatableToken,
}

impl FromStr for YdbConnectOptions {
    type Err = sqlx_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

fn default_tx_control() -> TransactionControl {
    TransactionControl { 
        commit_tx: true, 
        tx_selector: Some(TxSelector::BeginTx(TransactionSettings { 
            //TODO: продумать разные варианты TxMode
            tx_mode: Some(TxMode::SerializableReadWrite(Default::default())) 
        }))
    }
}

impl ConnectOptions for YdbConnectOptions {
    type Connection = YdbConnection;

    fn from_url(url: &sqlx_core::Url) -> Result<Self, sqlx_core::Error> {
        todo!()
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, sqlx_core::Error>>
    where
        Self::Connection: Sized {
        let channel = self.endpoint.make_endpoint().connect_lazy();
        let inner = crate::YdbConnection::new(channel, self.db_name.clone(), self.creds.clone());
        let tx_control = default_tx_control();
        Box::pin(async move{
            Ok(YdbConnection { inner, options: self.clone(), tx_control })
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

    type Options = YdbConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), sqlx_core::Error>> {
        Box::pin(async move{
            self.inner.close_session().await?;
            Ok(())
        })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), sqlx_core::Error>> {
        self.inner.close_session_hard();
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

impl YdbConnection {
    pub async fn executor(&mut self) -> Result<YdbTransaction<'_, UpdatableToken>, YdbError> {
        let table = self.inner.table().await?;
        Ok(YdbTransaction::new(table, self.tx_control.clone()))
    }
}

pub struct YdbTransactionManager;

impl TransactionManager for YdbTransactionManager {
    type Database = Ydb;

    fn begin(conn: &mut YdbConnection) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {Box::pin(async{
        let tx_settings = Some(TransactionSettings{tx_mode: Some(TxMode::SerializableReadWrite(Default::default()))});
        let response = conn.inner.table().await?.begin_transaction(BeginTransactionRequest{tx_settings, ..Default::default()}).await?;
        let tx_id = response.into_inner().result().map_err(YdbError::from)?.tx_meta.unwrap().id;
        conn.tx_control = TransactionControl{commit_tx: false, tx_selector: Some(TxSelector::TxId(tx_id))};
        Ok(())
    })}

    fn commit(conn: &mut YdbConnection) -> BoxFuture<'_, Result<(), sqlx_core::Error>> { Box::pin(async { 
        conn.executor().await?.commit_inner().await?;
        conn.tx_control = default_tx_control();
        Ok(())
    })}

    fn rollback(conn: &mut YdbConnection) -> BoxFuture<'_, Result<(), sqlx_core::Error>> { Box::pin(async {
        conn.executor().await?.rollback_inner().await?;
        conn.tx_control = default_tx_control();
        Ok(())
    })}

    fn start_rollback(conn: &mut YdbConnection) {
        conn.tx_control = default_tx_control();
        log::error!("start_rollback method is unimplemented");
    }
}