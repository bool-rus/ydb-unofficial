#![allow(dead_code)]

use std::{env, time::Duration};

//use ydb_grpc::ydb_proto::{discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, ListEndpointsRequest}, table::{v1::table_service_client::TableServiceClient, CreateSessionRequest}};
use exper::YdbResponse;

use crate::generated::ydb::table::{ExecuteDataQueryRequest, query::Query, self, TransactionControl, TransactionSettings, transaction_settings::TxMode, transaction_control::TxSelector};

use self::client::YdbService;

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
    //let response = client.list_endpoints(ListEndpointsRequest{database: db_name.into(), ..Default::default()}).await.unwrap();
    //let payload = response.into_inner().payload().unwrap();
    //println!("payload: {payload:?}");

    let table_client = service.clone().table();
    //let mut table_client = TableServiceClient::connect("").await.unwrap();
    {
        use client::StartSession;

        let tx_settings = TransactionSettings{tx_mode: Some(TxMode::SerializableReadWrite(Default::default()))};
        let selector = TxSelector::BeginTx(tx_settings);
        let query = "SELECT 1+1 as sum, 2*2 as mul";
        let mut session = table_client.start_session().await.unwrap();
        let x = session.execute_data_query(ExecuteDataQueryRequest{
            tx_control: Some(TransactionControl{commit_tx: false, tx_selector: Some(selector.clone())}),
            query: Some(table::Query{query: Some(Query::YqlText(query.into()))}),
            ..Default::default()
        }).await.unwrap();

        println!("\nresponse: {x:?}\n");
        let payload = x.into_inner().payload();
        println!("payload: {:?}", payload);

        let tx_id = payload.unwrap().tx_meta.unwrap().id;
        let selector = TxSelector::TxId(tx_id);
        
        let x = session.execute_data_query(ExecuteDataQueryRequest{
            tx_control: Some(TransactionControl{commit_tx: false, tx_selector: Some(selector.clone())}),
            query: Some(table::Query{query: Some(Query::YqlText(query.into()))}),
            ..Default::default()
        }).await.unwrap();

        println!("\nx: {x:?}");
        let payload = x.into_inner().payload();
        println!("\npayload: {payload:?}")
        //session.query("SELECT 1+1 as sum, 2*2 as mul".into()).await.unwrap();
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    
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
    let _s = baz.foo();
}


