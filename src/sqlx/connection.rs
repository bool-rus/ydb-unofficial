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
use crate::client::{YdbEndpoint, TableClientWithSession};

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

impl YdbConnectOptions {
    pub fn with_creds(mut self, creds: UpdatableToken) -> Self {
        self.creds = creds;
        self
    }
}

impl FromStr for YdbConnectOptions {
    type Err = sqlx_core::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = sqlx_core::Url::try_from(s).map_err(|e|sqlx_core::Error::Configuration(format!("Cannot parse connection string as url: {e}").into()))?;
        Self::from_url(&url)
    }
}

#[test]
fn test_conn_options_from_str() {
    let options = YdbConnectOptions::from_str("ydbs://ydb.serverless.yandexcloud.net:2135/ru-central1/some-anfslndundf908/234ndfnsdkjf").unwrap();
    assert!(options.endpoint.ssl);
    assert_eq!(options.endpoint.host, "ydb.serverless.yandexcloud.net");
    assert_eq!(options.endpoint.port, 2135);
    assert_eq!(options.db_name.as_bytes(), "/ru-central1/some-anfslndundf908/234ndfnsdkjf".as_bytes());
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
        use sqlx_core::Error::Configuration as ConfErr;
        let ssl = match url.scheme() {
            "ydb" | "grpc" => false,
            "ydbs" | "grpcs" => true,
            _ => return Err(ConfErr("Unknown scheme".into()))
        };
        let host = url.host_str().ok_or(ConfErr("no host".into()))?.into();
        let port = url.port().ok_or(ConfErr("no port".into()))?;
        let db_name = url.path().try_into().map_err(|e|ConfErr(format!("cannot parse database name: {e}").into())).unwrap();
        let endpoint = YdbEndpoint { ssl, host, port, load_factor: 0.0 };
        let mut creds = UpdatableToken::new("".try_into().unwrap());
        for (k,v) in url.query_pairs() {
            match k.as_ref() {
                "token" => {
                    let token = v.as_ref().try_into().map_err(|e|ConfErr(format!("cannot parse database name: {e}").into()))?;
                    creds = UpdatableToken::new(token);
                    break;
                }
                #[cfg(feature = "auth-sa")]
                "sa-key" => {
                    let file = std::fs::File::open(v.as_ref()).map_err(|e|ConfErr(format!("cannot open sa file: {e}").into()))?;
                    use crate::auth::sa::*;
                    let key: ServiceAccountKey = serde_json::from_reader(file).map_err(|e|ConfErr(format!("cannot parse sa file: {e}").into()))?;
                    creds = tokio::task::block_in_place(||{
                        futures::executor::block_on(async {
                            ServiceAccountCredentials::create(key).await
                        })
                    })
                    .map_err(YdbError::from)?.into();
                    break;
                }
                _ => {}
            }
        };
        Ok(Self{endpoint, db_name, creds})
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, sqlx_core::Error>>
    where
        Self::Connection: Sized { //TODO: реализовать подключение к разным эндпойнтам из discovery (чтобы pool подключался как надо)
        let channel = self.endpoint.make_endpoint().connect_lazy();
        let mut inner = crate::YdbConnection::new(channel, self.db_name.clone(), self.creds.clone());
        let tx_control = default_tx_control();
        Box::pin(async move {
            let _ = inner.table().await?;
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

    fn ping(&mut self) -> BoxFuture<'_, Result<(), sqlx_core::Error>> { Box::pin( async {
        self.inner.table() //коль скоро мы в асинхронной функции, можем и восстановить сессию. Поэтому table()
            .await?.keep_alive(Default::default()).await?;
        Ok(())
    })}

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
    /// Retrieve DML executor, that can select/insert/update values in existing tables, but cannot modify their definitions
    pub fn executor(&mut self) -> Result<YdbTransaction<'_, UpdatableToken>, YdbError> {
        let tx_control = self.tx_control.clone();
        let table = self.scheme_executor()?;
        Ok(YdbTransaction::new(table, tx_control))
    }
    /// Retrieve DDL executor, that makes operations on tables (create, delete, replace tables/indexes/etc).
    /// Note that DDL executor cannot fetch results, prepare and describe (never can used in sqlx macro). Parameter binding also unavailable
    pub fn scheme_executor(&mut self) -> Result<TableClientWithSession<'_, UpdatableToken>, YdbError> {
        self.inner.table_if_ready().ok_or(YdbError::NoSession)
    }
    /// Reconnect to Ydb if received [YdbError::NoSession] received
    /// Sometimes Ydb service can invalidate connection with Session. An if you use single connection, you need to reconnect them
    pub async fn reconnect(&mut self) -> Result<(), sqlx_core::Error> {
        let conn = self.options.connect().await?;
        *self = conn;
        Ok(())
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
        conn.executor()?.commit_inner().await?;
        conn.tx_control = default_tx_control();
        Ok(())
    })}

    fn rollback(conn: &mut YdbConnection) -> BoxFuture<'_, Result<(), sqlx_core::Error>> { Box::pin(async {
        conn.executor()?.rollback_inner().await?;
        conn.tx_control = default_tx_control();
        Ok(())
    })}

    fn start_rollback(conn: &mut YdbConnection) {
        conn.tx_control = default_tx_control();
        log::error!("start_rollback method is unimplemented");
    }
}