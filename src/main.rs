use std::{future::Future, env, time::Duration};

use tonic::{transport::{Certificate, ClientTlsConfig, Channel}, codegen::InterceptedService, service::Interceptor};
//use ydb_grpc::ydb_proto::{discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, ListEndpointsRequest}, table::{v1::table_service_client::TableServiceClient, CreateSessionRequest}};
use exper::YdbResponse;
use generated::google::protobuf::Any;

use crate::generated::{ydb::{discovery::{ListEndpointsRequest, ListEndpointsResponse, v1::MyStruct}, table::{v1::table_service_client::TableServiceClient, CreateSessionRequest, DeleteSessionRequest, ExecuteDataQueryRequest, query::Query, self, TransactionControl, TransactionSettings, transaction_settings::TxMode, OnlineModeSettings, transaction_control::TxSelector, CreateSessionResponse}}, DiscoveryServiceClient};

use self::client::{DBInterceptor, Credentials, YdbService};

mod pool;
mod client;
mod exper;
mod generated;


#[tokio::main]
pub async fn main() {
    println!("hello world");
    let url = "grpcs://ydb.serverless.yandexcloud.net:2135";
    let db_name = "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f";
    //let url = "grpcs://localhost:2135";
    //let db_name = "/local";
    let creds = env::var("TOKEN").unwrap();
    //println!("tls config: {tls_config:?}");
    let ep = client::create_endpoint(url.try_into().unwrap());
    let channel = ep.connect().await.unwrap();
    let service = YdbService::new(channel, db_name.try_into().unwrap(), creds.to_owned());

    //client::Client::new(url, db_name, creds.to_owned()).await.unwrap();

    let mut client = service.clone().discovery();
    //let mut client = DiscoveryServiceClient::connect("test").await.unwrap();
    let response = client.list_endpoints(ListEndpointsRequest{database: db_name.into(), ..Default::default()}).await.unwrap();
    let payload = response.into_inner().payload().unwrap();
    println!("payload: {payload:?}");

    let table_client = service.clone().table();
    //let mut table_client = TableServiceClient::connect("").await.unwrap();
    {
        use client::StartSession;
        let mut session = table_client.start_session().await.unwrap();
        session.query("SELECT 1+1 as sum, 2*2 as mul".into()).await.unwrap();
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    
}


async fn with_session<C: Credentials, Fut: Future, F: FnMut(String)->Fut>(service: YdbService<C>, mut fun: F) -> Fut::Output {
    let mut table_client = TableServiceClient::new(service);
    let session: CreateSessionResponse = table_client.create_session(CreateSessionRequest::default()).await.unwrap().into_inner();
    let session_id = session.payload().unwrap().session_id;
    let result = fun(session_id.clone()).await;
    table_client.delete_session(DeleteSessionRequest{session_id, ..Default::default()}).await.unwrap();
    result
}


trait Foo {type Inner;}
trait Bar {type Inner;}
impl Foo for i32 {type Inner = i32;}
impl Bar for i32 {type Inner = i32;}
struct Baz<T>(T);
impl<T> Baz<T> where T: Foo, T::Inner: Bar,
<T::Inner as Bar>::Inner: Sized
{
    pub fn new(obj: T) -> Self {Self(obj)}
    pub fn foo(&self) -> String {"foobazz".to_owned()}
}

fn test() {
    let baz = Baz::new(1);
    let s = baz.foo();
}


