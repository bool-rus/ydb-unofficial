use crate::YdbError;

use table::*;

use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Endpoint, Channel, Uri};

use crate::payload::YdbResponseWithResult;
use crate::generated::ydb::discovery::v1::DiscoveryServiceClient;
use crate::generated::ydb::table::transaction_control::TxSelector;
use crate::generated::ydb::table::{TransactionSettings, OnlineModeSettings, ExecuteDataQueryRequest, TransactionControl, self, CreateSessionRequest, DeleteSessionRequest};
use crate::generated::ydb::table::transaction_settings::TxMode;
use crate::generated::ydb::table::v1::table_service_client::TableServiceClient;
pub type AsciiValue = tonic::metadata::MetadataValue<tonic::metadata::Ascii>;

pub fn create_endpoint(uri: Uri) -> Endpoint {
    let mut res = Endpoint::from(uri);
    if matches!(res.uri().scheme_str(), Some("grpcs")) {
        res = res.tls_config(tonic::transport::ClientTlsConfig::new()).unwrap()
    };
    res.tcp_keepalive(Some(std::time::Duration::from_secs(15)))
}


pub trait Credentials: Clone + Send + 'static {
    fn token(&self) -> AsciiValue;
}

impl Credentials for String {
    fn token(&self) -> AsciiValue {
        self.clone().try_into().unwrap()
    }
}

#[ctor::ctor]
static BUILD_INFO: AsciiValue = concat!("ydb-unofficial/", env!("CARGO_PKG_VERSION")).try_into().unwrap();

#[derive(Clone, Debug)]
pub struct DBInterceptor<C: Clone> {
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

pub struct YdbService<C: Credentials> {
    inner: InterceptedService<Channel, DBInterceptor<C>>,
    session_id: Option<String>,
}

use tower::Service as Service1;

impl<C: Credentials> Service1<tonic::codegen::http::Request<tonic::body::BoxBody>> for YdbService<C> {
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

impl<C: Credentials> YdbService<C> {
    pub fn new(channel: Channel, db_name: AsciiValue, creds: C) -> Self {
        let interceptor = DBInterceptor {db_name, creds};
        let inner = tower::ServiceBuilder::new()
            .layer(tonic::service::interceptor(interceptor))
            .layer_fn(|x|x)
            .service(channel);
        YdbService{inner, session_id: None}
    }
    pub fn discovery(&mut self) -> DiscoveryServiceClient<&mut Self> {
        DiscoveryServiceClient::new(self)
    }
    pub async fn table(&mut self) -> Result<TableClientWithSession<C>, YdbError> {
        let session_id = if let Some(session_id) = self.session_id.clone() {
            session_id
        } else {
            let mut client = TableServiceClient::new(&mut self.inner);
            let response = client.create_session(CreateSessionRequest::default()).await?;
            let session_id = response.into_inner().result()?.session_id;
            log::debug!("Session created: {session_id}");
            self.session_id = Some(session_id.clone());
            session_id
        };
        let client = TableServiceClient::new(self);
        Ok(TableClientWithSession { session_id, client })
    }
}

impl<C: Credentials> Drop for YdbService<C> {
    fn drop(&mut self) {
        if let Some(session_id) = self.session_id.clone() {
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
        log::debug!("YdbService closed");
    }
}

pub struct TableClientWithSession<'a, C: Credentials> {
    session_id: String,
    client: TableServiceClient<&'a mut YdbService<C>>,
}

macro_rules! delegate {
    (with $field:ident : $(fn $fun:ident($arg:ty) -> $ret:ty;)+) => { $(
        pub async fn $fun(&mut self, mut req: $arg) -> Result<tonic::Response<$ret>, YdbError> {
            req.$field = self.$field.clone();
            let result = self.client.$fun(req).await?;
            let status = result.get_ref().operation.as_ref().ok_or(YdbError::EmptyResponse).unwrap().status();
            use crate::generated::ydb::status_ids::StatusCode;
            use crate::error::ErrWithOperation;
            match status {
                StatusCode::Success => Ok(result),
                _ => Err(YdbError::Ydb(ErrWithOperation(result.into_inner().operation.unwrap()))),
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
    }
    pub async fn stream_read_table(&mut self, mut req: ReadTableRequest) -> Result<tonic::Response<tonic::codec::Streaming<ReadTableResponse>>, tonic::Status> {
        req.session_id = self.session_id.clone();
        self.client.stream_read_table(req).await
    }
}


pub struct YdbTransaction<'a, C: Credentials> {
    tx_control: Option<TransactionControl>,
    client: TableClientWithSession<'a, C>,
}

impl<'a, C: Credentials> YdbTransaction<'a, C> {
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
    pub async fn execute_data_query(&mut self, mut req: ExecuteDataQueryRequest) -> Result<tonic::Response<ExecuteDataQueryResponse>,YdbError> {
        req.tx_control = self.tx_control.clone();
        self.client.execute_data_query(req).await
    }
}
