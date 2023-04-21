
use std::error::Error;
use std::future::Future;
use async_trait::async_trait;
use prost::Message;
use table::*;

use tonic::codegen::{InterceptedService, http};
use tonic::service::Interceptor;
use tonic::transport::{Endpoint, Channel, Uri};

use crate::exper::YdbResponse;
use crate::generated::ydb::discovery::v1::DiscoveryServiceClient;
use crate::generated::ydb::discovery::{ListEndpointsResult, ListEndpointsRequest};
use crate::generated::ydb::table::query::Query;
use crate::generated::ydb::table::transaction_control::TxSelector;
use crate::generated::ydb::table::{TransactionSettings, OnlineModeSettings, ExecuteDataQueryRequest, TransactionControl, self, CreateSessionRequest, DeleteSessionRequest};
use crate::generated::ydb::table::transaction_settings::TxMode;
use crate::generated::ydb::table::v1::table_service_client::TableServiceClient;
//use ydb_grpc::ydb_proto::discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, WhoAmIResponse, ListEndpointsRequest, WhoAmIResult, ListEndpointsResult};

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

#[derive(Clone)]
pub struct YdbService<C: Credentials>(InterceptedService<Channel, DBInterceptor<C>>);

use tonic::client::GrpcService as Service;
use tonic::body::BoxBody as Body;

impl<C: Credentials> Service<Body> for YdbService<C> {
    type ResponseBody = Body;

    type Error = tonic::transport::Error;

    type Future = tonic::service::interceptor::ResponseFuture<tonic::transport::channel::ResponseFuture>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, request: tonic::codegen::http::Request<tonic::body::BoxBody>) -> Self::Future {
        self.0.call(request)
    }
}

impl<C: Credentials> YdbService<C> {
    pub fn new(channel: Channel, db_name: AsciiValue, creds: C) -> Self {
        let interceptor = DBInterceptor {db_name, creds};
        let service = tower::ServiceBuilder::new()
            .layer(tonic::service::interceptor(interceptor))
            .layer_fn(|x|x)
            .service(channel);
        YdbService(service)
    }
    pub fn discovery(self) -> DiscoveryServiceClient<Self> {
        DiscoveryServiceClient::new(self)
    }
    pub fn table(self) -> TableServiceClient<Self> {
        TableServiceClient::new(self)
    }
}

type YdbError = Box<dyn Error>;

pub struct WithSession<C: Credentials> {
    session_id: String,
    client: TableServiceClient<YdbService<C>>
}

#[async_trait]
pub trait StartSession<C: Credentials> {
    async fn start_session(self) -> Result<WithSession<C>, YdbError>;
}
#[async_trait]
impl<C: Credentials> StartSession<C> for TableServiceClient<YdbService<C>> {
    async fn start_session(mut self) -> Result<WithSession<C>, YdbError> {
        let response = self.create_session(CreateSessionRequest::default()).await?;
        let session_id = response.into_inner().payload().unwrap().session_id;
        Ok(WithSession{session_id, client: self})
    }
}

impl<C: Credentials> Drop for WithSession<C> {
    fn drop(&mut self) {
        let WithSession {session_id, client} = self;
        let session_id = session_id.clone();
        println!("session: {session_id}");
        let mut client = client.clone();
        tokio::spawn(async move {
            let res = client.delete_session(DeleteSessionRequest{session_id, ..Default::default()}).await;
            if let Some(e) = res.err() {
                println!("Error on closing session: {e}");
            } else {
                println!("Session closed");
            }
        });
    }
}


macro_rules! delegate {
    (with $field:ident : $(fn $fun:ident($arg:ty) -> $ret:ty;)+) => { $(
        pub async fn $fun(&mut self, mut req: $arg) -> Result<tonic::Response<$ret>, tonic::Status> {
            req.$field = self.$field.clone();
            self.client.$fun(req).await
        }
    )+} 
}

impl <C: Credentials + Send> WithSession<C> {
    pub async fn query(&mut self, query: String) -> Result<(), YdbError> {
        let tx_settings = TransactionSettings{tx_mode: Some(TxMode::OnlineReadOnly(OnlineModeSettings{allow_inconsistent_reads: true}))};
        let selector = TxSelector::BeginTx(tx_settings);
        let x = self.execute_data_query(ExecuteDataQueryRequest{
            tx_control: Some(TransactionControl{commit_tx: true, tx_selector: Some(selector.clone())}),
            query: Some(table::Query{query: Some(Query::YqlText(query.into()))}),
            ..Default::default()
        }).await?;
        println!("\nresponse: {x:?}\n");
        //let status = x.into_inner().operation.unwrap().status();
        let result_sets = x.into_inner().payload().unwrap().result_sets;
        for rs in result_sets {
            for row in rs.rows {
                for item in row.items {
                    println!("item: {item:?}");
                }
            }
        }
    
        Ok(())
    }
    delegate!{ with session_id:
        fn execute_data_query(ExecuteDataQueryRequest) -> ExecuteDataQueryResponse;
        fn prepare_data_query(PrepareDataQueryRequest) -> PrepareDataQueryResponse;
    }
}


