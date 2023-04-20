
use std::error::Error;
use prost::Message;

use tonic::codegen::{InterceptedService, http};
use tonic::service::Interceptor;
use tonic::transport::{Endpoint, Channel, Uri};

use crate::generated::ydb::discovery::v1::DiscoveryServiceClient;
use crate::generated::ydb::discovery::{ListEndpointsResult, ListEndpointsRequest};
//use ydb_grpc::ydb_proto::discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, WhoAmIResponse, ListEndpointsRequest, WhoAmIResult, ListEndpointsResult};

pub type AsciiValue = tonic::metadata::MetadataValue<tonic::metadata::Ascii>;


pub fn create_ydb_service<C: Credentials>(channel: Channel, db_name: String, creds: C) -> YdbService<C> {
    let db_name = db_name.try_into().unwrap();
    let interceptor = DBInterceptor {db_name, creds};
    let service = tower::ServiceBuilder::new()
        .layer(tonic::service::interceptor(interceptor))
        .layer_fn(|x|x)
        .service(channel);
    YdbService(service)
}



pub fn create_endpoint(uri: Uri) -> Endpoint {
    let mut res = Endpoint::from(uri);
    if matches!(res.uri().scheme_str(), Some("grpcs")) {
        res = res.tls_config(tonic::transport::ClientTlsConfig::new()).unwrap()
    };
    res.tcp_keepalive(Some(std::time::Duration::from_secs(15)))
}


#[derive(Debug)]
pub struct Client<C: Credentials + Clone> {
    endpoints: Vec<Endpoint>,
    channel: Channel,
    interceptor: DBInterceptor<C>,
}

impl<C: Credentials + Clone> Client<C> {
    pub async fn new<E: TryInto<Endpoint>>(endpoint: E, db_name: &str, creds: C) -> Result<Self, Box<dyn Error>> where E::Error : std::error::Error + 'static {
        let db_name = db_name.try_into()?;
        let endpoint: Endpoint = endpoint.try_into()?;
        let endpoint = endpoint.tcp_keepalive(Some(std::time::Duration::from_secs(15)));
            //.tls_config(tonic::transport::ClientTlsConfig::new())?;
        let channel = endpoint.connect().await?;
        let interceptor = DBInterceptor {db_name, creds};
        let result = Self::list_endpoints(channel.clone(), interceptor.clone()).await?;
        let mut endpoints = Vec::with_capacity(result.endpoints.len());
        for e in result.endpoints {
            println!("endpoint: {e:?}");
            let endpoint: Endpoint = e.address.try_into()?;
            endpoints.push(Self::correct_endpoint(endpoint));
        }
        let me = Self{endpoints, channel, interceptor};
        Ok(me)
    }
    fn correct_endpoint(endpoint: Endpoint) -> Endpoint {
        match endpoint.uri().scheme_str() {
            Some("grpcs") => endpoint.tls_config(tonic::transport::ClientTlsConfig::new()).unwrap(),
            _ => endpoint
        }.tcp_keepalive(Some(std::time::Duration::from_secs(15)))
    }
    async fn list_endpoints(channel: Channel, interceptor: DBInterceptor<C>) -> Result<ListEndpointsResult, Box<dyn Error>> {
        let database = interceptor.db_name();
        let req = ListEndpointsRequest {
            database,
            ..ListEndpointsRequest::default()
        };
        println!("req: {req:?}\n");
        let mut discovery = DiscoveryServiceClient::with_interceptor(channel, interceptor);
        let response = discovery.list_endpoints(req).await?.into_inner();
        ListEndpointsResult::decode(response.operation.unwrap().result.unwrap().value.as_slice()).map_err(Into::into)
    }
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

impl<C: Clone> DBInterceptor<C> {
    fn db_name(&self) -> String {
        self.db_name.to_str().unwrap().to_owned()
    }
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