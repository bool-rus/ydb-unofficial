
use std::error::Error;
use prost::Message;

use tonic::codegen::{InterceptedService, http};
use tonic::service::Interceptor;
use tonic::transport::{Endpoint, Channel, Uri};

use crate::generated::ydb::discovery::v1::DiscoveryServiceClient;
use crate::generated::ydb::discovery::{ListEndpointsResult, ListEndpointsRequest};
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


pub trait Credentials: Clone {
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
        println!("headers added");
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