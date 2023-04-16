use std::{sync::{Arc, RwLock}, ops::Deref, error::Error, future::Future};
use prost::Message;

use tonic::{transport::{Endpoint, Uri, channel::ResponseFuture, Channel}, codegen::InterceptedService, service::Interceptor};
use ydb_grpc::ydb_proto::{table::v1::table_service_client::TableServiceClient, discovery::{self, v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, WhoAmIResponse, ListEndpointsRequest, WhoAmIResult, ListEndpointsResult}};

pub type AsciiValue = tonic::metadata::MetadataValue<tonic::metadata::Ascii>;


#[tokio::main]
pub async fn main() {
    println!("hello world");
    let client = Client::new(
        "grpcs://ydb.serverless.yandexcloud.net:2135/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f", 
        "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f",
        "t1.9euelZqUzcuOmZnMks-MxpKYjI-Km-3rnpWamcmNip2Tk46RxpyZlpuTyo_l8_d6URBe-e94Ek81_d3z9zoADl7573gSTzX9.ca2UcJS5Vnjqe7EvvV45C5mF0xQxyXXfOaUodSQtKJitDMMA4zuW7HdLFmPhX1GSp15ZSXmKC5WdWZqknf3DBw".to_owned(), 
    ).await.unwrap();
    println!("client: {client:?}");
    let ami= client.whoami().await.unwrap();
    println!("i am: {ami:?}");
    //let endpoints = client.list_endpoints().await.unwrap();
    //for ep in endpoints.endpoints {
    //    println!("endpoint: {}, services: {:?}", ep.address, ep.service);
    //}
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
        let endpoint = endpoint.tcp_keepalive(Some(std::time::Duration::from_secs(15)))
            .tls_config(tonic::transport::ClientTlsConfig::new())?;
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
    pub async fn whoami(&self) -> Result<WhoAmIResponse, Box<dyn Error>> {
        let channel = self.channel.clone();
        
        let mut discovery = DiscoveryServiceClient::with_interceptor(channel, self.interceptor.clone());
        let response: tonic::Response<WhoAmIResponse> = discovery.who_am_i(WhoAmIRequest {include_groups: true}).await.unwrap();
        let response = response.into_inner();
        let any = response.operation.as_ref().unwrap().result.as_ref().unwrap();
        let result = WhoAmIResult::decode(any.value.as_slice());
        println!("result: {result:?}");
        Ok(response)
    }
}

pub trait Credentials {
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

impl<C: Credentials + Clone> Interceptor for DBInterceptor<C> {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let headers = request.metadata_mut();
        headers.insert("x-ydb-database", self.db_name.clone());
        headers.insert("x-ydb-sdk-build-info", BUILD_INFO.clone());
        headers.insert("x-ydb-auth-ticket", self.creds.token());
        Ok(request)    
    }
}