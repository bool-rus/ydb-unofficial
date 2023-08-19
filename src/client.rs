//! Common Ydb client, that wraps GRPC(s) transport with needed headers
//! 
//! # Examples
//! ``` rust
//! # #[tokio::main]
//! # async fn main() {
//!     let url = std::env::var("YDB_URL").expect("YDB_URL not set");
//!     let db_name = std::env::var("DB_NAME").expect("DB_NAME not set");
//!      let creds = std::env::var("DB_TOKEN").expect("DB_TOKEN not set");
//! 
//!     // how you can create service
//!     let endpoint = ydb_unofficial::client::create_endpoint(url.try_into().unwrap());
//!     let channel = endpoint.connect().await.unwrap();
//!     use ydb_unofficial::YdbConnection;
//!     let mut service = YdbConnection::new(channel, db_name.as_str().try_into().unwrap(), creds);
//! 
//!     // how to use it, e.g. use discovery service:
//!     use ydb_unofficial::generated::ydb::discovery::ListEndpointsRequest;
//!     let endpoints_response = service.discovery().list_endpoints(
//!         ListEndpointsRequest{
//!             database: db_name.into(), 
//!             ..Default::default()
//!         }
//!     ).await.unwrap();
//!     
//!     // how you can parse response to invoke result with YdbResponseWithResult trait
//!     use ydb_unofficial::YdbResponseWithResult;
//!     let endpoints_result = endpoints_response.get_ref().result().unwrap();
//!     assert!(endpoints_result.endpoints.len() > 0);
//! 
//!     // now to use table operations
//!     use ydb_unofficial::generated::ydb::table;
//!     
//! # }
//! ```
use super::*;
use std::sync::{Arc, RwLock};
use error::YdbError;
use auth::Credentials;

use table::*;

use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Endpoint, Channel, Uri};

use payload::YdbResponseWithResult;
use generated::ydb::discovery::v1::discovery_service_client::DiscoveryServiceClient;
use generated::ydb::table::transaction_control::TxSelector;
use generated::ydb::table::{TransactionSettings, ExecuteDataQueryRequest, TransactionControl, self, CreateSessionRequest, DeleteSessionRequest};
use generated::ydb::table::transaction_settings::TxMode;
use generated::ydb::table::v1::table_service_client::TableServiceClient;
use tower::Service;

#[derive(Debug, Clone)]
pub struct YdbEndpoint {
    pub ssl: bool,
    pub host: String,
    pub port: u16,
    pub load_factor: f32,
}

impl YdbEndpoint {
    pub fn scheme(&self) -> &'static str {
        if self.ssl { "grpcs" } else { "grpc" }
    }
    pub fn make_endpoint(&self) -> Endpoint {
        let uri: tonic::transport::Uri = format!("{}://{}:{}", self.scheme(), self.host, self.port).try_into().unwrap();
        let mut e = Endpoint::from(uri).tcp_keepalive(Some(std::time::Duration::from_secs(15)));
        if self.ssl {
            e = e.tls_config(Default::default()).unwrap()
        }
        e
    }
}

impl TryFrom<Uri> for YdbEndpoint {
    type Error = String;

    fn try_from(value: Uri) -> Result<Self, Self::Error> {
        //TODO: убрать дублирование
        let ssl = match value.scheme_str() {
            Some("grpc") => false,
            Some("grpcs") => true,
            _ => return Err("Unknown protocol".to_owned()),
        };
        let host = value.host().ok_or("no host")?.to_owned();
        let port = value.port_u16().ok_or("no port")?;
        let load_factor = 0.0;
        Ok(Self {ssl, host, port, load_factor})
    }
}


/// Creates endpoint from uri
/// If protocol is `grpcs`, then creates [`tonic::transport::ClientTlsConfig`] and applies to [`Endpoint`]
///
/// # Arguments
///
/// * `uri` - An [`Uri`] of endpoint
///
/// # Examples
///
/// ```
/// use ydb_unofficial::client;
/// let url = "grpcs://ydb.serverless.yandexcloud.net:2135";
/// let enpoint = client::create_endpoint(url.try_into().unwrap());
/// ```
pub fn create_endpoint(uri: Uri) -> Endpoint {
    let mut res = Endpoint::from(uri);
    if matches!(res.uri().scheme_str(), Some("grpcs")) {
        res = res.tls_config(tonic::transport::ClientTlsConfig::new()).unwrap()
    };
    res.tcp_keepalive(Some(std::time::Duration::from_secs(15)))
}


//#[allow(non_upper_case_globals)]
#[ctor::ctor]
static BUILD_INFO: AsciiValue = concat!("ydb-unofficial/", env!("CARGO_PKG_VERSION")).try_into().unwrap();

#[derive(Clone, Debug)]
struct DBInterceptor<C: Clone> {
    db_name: AsciiValue,
    creds: C
}

impl<C: Credentials> Interceptor for DBInterceptor<C> {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let headers = request.metadata_mut();
        headers.insert("x-ydb-database", self.db_name.clone());
        headers.insert("x-ydb-sdk-build-info", BUILD_INFO.clone());
        headers.insert("x-ydb-auth-ticket", self.creds.token());
        Ok(request)    
    }
}

/// Ydb connection implementation, that pass database name and auth data to grpc channel
#[derive(Debug)]
pub struct YdbConnection<C: Credentials> {
    inner: InterceptedService<Channel, DBInterceptor<C>>,
    session_id: Arc<RwLock<Option<String>>>,
    tx_control: Option<TransactionControl>,
}


impl<C: Credentials> Service<tonic::codegen::http::Request<tonic::body::BoxBody>> for YdbConnection<C> {
    type Response = tonic::codegen::http::Response<tonic::body::BoxBody>;

    type Error = tonic::transport::Error;

    type Future = tonic::service::interceptor::ResponseFuture<tonic::transport::channel::ResponseFuture>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: tonic::codegen::http::Request<tonic::body::BoxBody>) -> Self::Future {
        self.inner.call(request)
    }
}

impl YdbConnection<String> {
    /// Creates connection from environment.
    /// It uses following env variables
    /// * YDB_URL - grpc-url to database host
    /// * DB_NAME - name of database connect to
    /// * DB_TOKEN - temporary token to access to database
    pub fn from_env() -> Self {
        use std::env::var;
        let url = var("YDB_URL").expect("YDB_URL not set");
        let db_name = var("DB_NAME").expect("DB_NAME not set");
        let creds = var("DB_TOKEN").expect("DB_TOKEN not set");
    
        let endpoint = create_endpoint(url.try_into().unwrap());
        let channel = endpoint.connect_lazy();
        YdbConnection::new(channel, db_name.as_str().try_into().unwrap(), creds)
    }
}

impl<C: Credentials> YdbConnection<C> {
    /// YdbConnection constructor
    /// 
    /// # Arguments
    /// * `channel` - transport channel to database (can be make from [`Endpoint`])
    /// * `db_name` - database name (you can get it from yandex cloud for example) in [`AsciiValue`] format
    /// * `creds` - some object, that implements [`Credentials`] (e.g. [`String`])
    /// 
    /// # Examples
    /// See [`self`]
    pub fn new(channel: Channel, db_name: AsciiValue, creds: C) -> Self {
        let interceptor = DBInterceptor {db_name, creds};
        let inner = tower::ServiceBuilder::new()
            .layer(tonic::service::interceptor(interceptor))
            .service(channel);
        let tx_control = Some(TransactionControl { 
            commit_tx: true, 
            tx_selector: Some(TxSelector::BeginTx(TransactionSettings { 
                //TODO: продумать разные варианты TxMode
                tx_mode: Some(TxMode::SerializableReadWrite(Default::default())) 
            }))
        });
        YdbConnection{inner, session_id: Arc::new(RwLock::new(None)), tx_control}
    }
    /// Creates discovery service client
    /// 
    /// # Examples
    /// ```rust
    /// # #[tokio::main] 
    /// # async fn main() {
    ///     let mut conn = ydb_unofficial::YdbConnection::from_env();
    ///     let db_name = std::env::var("DB_NAME").unwrap();
    ///     use ydb_unofficial::generated::ydb::discovery::ListEndpointsRequest;
    ///     let endpoints_response = conn.discovery().list_endpoints(
    ///         ListEndpointsRequest{
    ///             database: db_name.into(), 
    ///             ..Default::default()
    ///         }
    ///     ).await.unwrap();
    ///     // how you can parse response to invoke result with YdbResponseWithResult trait
    ///     use ydb_unofficial::YdbResponseWithResult;
    ///     let endpoints_result = endpoints_response.get_ref().result().unwrap();
    ///     assert!(endpoints_result.endpoints.len() > 0);
    /// # }
    /// ```
    pub fn discovery(&mut self) -> DiscoveryServiceClient<&mut Self> {
        DiscoveryServiceClient::new(self)
    }

    /// Creates session and returns [`TableClientWithSession`]
    /// # Examples
    /// ```rust
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let mut conn = ydb_unofficial::YdbConnection::from_env();
    ///     let mut table_client = conn.table().await.unwrap();
    ///     use ydb_unofficial::generated::ydb::table::*;
    ///     let keep_alive_response = table_client.keep_alive(KeepAliveRequest::default()).await.unwrap();
    ///     //..some another code
    /// # }
    /// ```
    pub async fn table(&mut self) -> Result<TableClientWithSession<C>, YdbError> {
        let session_id = if let Some(session_id) = self.session_id() {
            session_id
        } else {
            let mut client = TableServiceClient::new(&mut self.inner);
            let response = client.create_session(CreateSessionRequest::default()).await?;
            let session_id = response.into_inner().result()?.session_id;
            log::debug!("Session created: {session_id}");
            *self.session_id.write().unwrap() = Some(session_id.clone());
            session_id
        };
        let session_ref = self.session_id.clone();
        let client = TableServiceClient::new(self);
        Ok(TableClientWithSession {session_ref, session_id, client })
    }
    fn session_id(&self) -> Option<String> {
        self.session_id.read().unwrap().clone()
    }
    pub async fn close_session(&mut self) -> Result<(), YdbError> {
        delete_session(&self.session_id, self.inner.clone()).await?;
        Ok(())
    }
    #[doc(hidden)]
    pub fn close_session_hard(self) {
        *self.session_id.write().unwrap() = None;
    }
}


async fn delete_session<C: Credentials>(session_ref: &Arc<RwLock<Option<String>>>, service: InterceptedService<Channel, DBInterceptor<C>>)  -> Result<(), YdbError> {
    let session_id = session_ref.read().unwrap().clone();
    if let Some(session_id) = session_id {
        let mut client = TableServiceClient::new(service);
        let response = client.delete_session(DeleteSessionRequest{session_id, ..Default::default()}).await?;
        let code = response.get_ref().operation.as_ref().ok_or(YdbError::EmptyResponse)?.status();
        process_session_fail(code, session_ref);
    }
    *session_ref.write().unwrap() = None;
    Ok(())
}

impl<C: Credentials> Drop for YdbConnection<C> {
    fn drop(&mut self) {
        if let Some(session_id) = self.session_id() {
            let mut client = TableServiceClient::new(self.inner.clone());
            tokio::spawn(async move {
                let copy = session_id.clone();
                if let Err(e) = client.delete_session(DeleteSessionRequest{session_id, ..Default::default()}).await {
                    log::error!("Error on closing session ({copy}): {e}");
                } else {
                    log::debug!("Session closed: {copy}");
                }
            });
        }
        log::debug!("YdbConnection closed");
    }
}

/// [`TableServiceClient`] with active session.
/// for each method (that requires session_id) table client injects session_id field
/// Session may be invalid. In this case you need to recreate that client with [`YdbConnection::table`]
#[derive(Debug)]
pub struct TableClientWithSession<'a, C: Credentials> {
    //TODO: тут бы это все как-то покрасивее сделать, но из TableServiceClient YdbConnection не достать
    session_ref: Arc<RwLock<Option<String>>>, 
    session_id: String,
    client: TableServiceClient<&'a mut YdbConnection<C>>,
}

fn process_session_fail(
    code: crate::generated::ydb::status_ids::StatusCode, 
    session_ref: &Arc<RwLock<Option<String>>>,
) {
    use crate::generated::ydb::status_ids::StatusCode;
    match code {
        StatusCode::BadSession | StatusCode::SessionExpired | StatusCode::SessionBusy => {
            *session_ref.write().unwrap() = None;
        },
        _ => {}
    }
}

macro_rules! delegate {
    (with $field:ident : $(fn $fun:ident($arg:ty) -> $ret:ty;)+) => { $(
        pub async fn $fun(&mut self, mut req: $arg) -> Result<tonic::Response<$ret>, YdbError> {
            req.$field = self.$field.clone();
            let result = self.client.$fun(req).await?;
            let status = result.get_ref().operation.as_ref().ok_or(YdbError::EmptyResponse)?.status();
            use crate::generated::ydb::status_ids::StatusCode;
            use crate::error::ErrWithOperation;
            match status {
                StatusCode::Success => Ok(result),
                _ => {
                    process_session_fail(status, &self.session_ref);
                    Err(YdbError::Ydb(ErrWithOperation(result.into_inner().operation.unwrap())))
                },
            }
        }
    )+} 
}


impl <'a, C: Credentials + Send> TableClientWithSession<'a, C> {
    delegate!{ with session_id:
        fn create_table(CreateTableRequest) -> CreateTableResponse;
        fn drop_table(DropTableRequest) -> DropTableResponse;
        fn alter_table(AlterTableRequest) -> AlterTableResponse;
        fn copy_table(CopyTableRequest) -> CopyTableResponse;
        fn rename_tables(RenameTablesRequest) -> RenameTablesResponse;
        fn describe_table(DescribeTableRequest) -> DescribeTableResponse;
        fn execute_data_query(ExecuteDataQueryRequest) -> ExecuteDataQueryResponse;
        fn execute_scheme_query(ExecuteSchemeQueryRequest) -> ExecuteSchemeQueryResponse;
        fn explain_data_query(ExplainDataQueryRequest) -> ExplainDataQueryResponse;
        fn prepare_data_query(PrepareDataQueryRequest) -> PrepareDataQueryResponse;
        fn keep_alive(KeepAliveRequest) -> KeepAliveResponse;
        fn begin_transaction(BeginTransactionRequest) -> BeginTransactionResponse;
        fn commit_transaction(CommitTransactionRequest) -> CommitTransactionResponse;
        fn rollback_transaction(RollbackTransactionRequest) -> RollbackTransactionResponse;
        fn delete_session(DeleteSessionRequest) -> DeleteSessionResponse;
    }
    pub async fn stream_read_table(&mut self, mut req: ReadTableRequest) -> Result<tonic::Response<tonic::codec::Streaming<ReadTableResponse>>, tonic::Status> {
        req.session_id = self.session_id.clone();
        self.client.stream_read_table(req).await
    }
    pub async fn execute_data_query_with_tx(&mut self, mut req: ExecuteDataQueryRequest) -> ExecuteDataQueryResponse {
        req.tx_control = self.client.tx_control.clone();
        self.execute_data_query(req).await
    }
}

/// [`TableServiceClient`] with active session and transaction
pub struct YdbTransaction<'a, C: Credentials> {
    tx_control: Option<TransactionControl>,
    //TODO: переделать на &mut
    client: TableClientWithSession<'a, C>,
}

/// Transactions implementation for ydb.
/// # Examples
/// ```rust
/// # #[tokio::main]
/// # async fn main() {
///   let mut conn = ydb_unofficial::YdbConnection::from_env();
///   let mut transaction = ydb_unofficial::YdbTransaction::create(conn.table().await.unwrap()).await.unwrap(); 
///   use ydb_unofficial::generated::ydb::table::query::Query;
///   let req = ydb_unofficial::generated::ydb::table::ExecuteDataQueryRequest{
///     query: Some(ydb_unofficial::generated::ydb::table::Query{
///        query: Some(Query::YqlText("SELECT 1+1 as sum, 2*2 as mul".into()))
///     }),
///      ..Default::default()
///   };
///   let result = transaction.execute_data_query(req).await.unwrap();
///   transaction.commit().await;
/// # }
/// ```
impl<'a, C: Credentials> YdbTransaction<'a, C> {
    /// Method that just creates ReadWrite transaction 
    pub async fn create(mut client: TableClientWithSession<'a, C>) -> Result<YdbTransaction<'a, C>, crate::error::YdbError> {
        let tx_settings = Some(TransactionSettings{tx_mode: Some(TxMode::SerializableReadWrite(Default::default()))});
        let response = client.begin_transaction(BeginTransactionRequest{tx_settings, ..Default::default()}).await?;
        let tx_id = response.into_inner().result()?.tx_meta.unwrap().id;
        let tx_control = Some(TransactionControl{commit_tx: false, tx_selector: Some(TxSelector::TxId(tx_id))});
        Ok(Self {tx_control, client})
    }
    fn invoke_tx_id(&self) -> String {
        if let TxSelector::TxId(tx_id) = self.tx_control.clone().unwrap().tx_selector.unwrap() {
            tx_id
        } else {
            panic!("looks like a bug")
        }
    }
    async fn commit_inner(&mut self) ->  Result<CommitTransactionResult, YdbError> {
        let tx_id = self.invoke_tx_id();
        let response = self.client.commit_transaction(CommitTransactionRequest {tx_id, ..Default::default()}).await?;
        let result = response.into_inner().result()?; //что там может быть полезного?
        Ok(result)
    }
    /// Commit transaction, drop transaction with commit
    //TODO: какое-то упоротое возвращаемое значение
    pub async fn commit(mut self) -> (TableClientWithSession<'a, C>, Result<CommitTransactionResult, YdbError>) {
        let result = self.commit_inner().await;
        (self.client, result)
    }
    async fn rollback_inner(&mut self) -> Result<(), YdbError> {
        let tx_id = self.invoke_tx_id();
        self.client.rollback_transaction(RollbackTransactionRequest {tx_id, ..Default::default()}).await?;
        Ok(())
    }
    pub async fn rollback(mut self) -> (TableClientWithSession<'a, C>, Result<(), YdbError> ) {
        let result = self.rollback_inner().await;
        (self.client, result)
    }
    /// you can execute multiple query requests in transaction
    /// transaction data will inject for each request
    pub async fn execute_data_query(&mut self, mut req: ExecuteDataQueryRequest) -> Result<tonic::Response<ExecuteDataQueryResponse>,YdbError> {
        req.tx_control = self.tx_control.clone();
        self.client.execute_data_query(req).await
    }
}
