#![allow(dead_code)]
use std::error::Error;
use std::{env, time::Duration};

use tokio::sync::futures;
use ydb_unofficial::generated::ydb::table::{ExecuteScanQueryRequest, ExecuteSchemeQueryRequest};
//use ydb_grpc::ydb_proto::{discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, ListEndpointsRequest}, table::{v1::table_service_client::TableServiceClient, CreateSessionRequest}};
use ydb_unofficial::{YdbResponseWithResult, generated::ydb::r#type::PrimitiveTypeId};
use ydb_unofficial::pool::YdbPoolBuilder;
use tonic::transport::Uri;
use ydb_unofficial::{pool::ConnectionManager, YdbError, generated::ydb::{table::{CreateTableRequest, ColumnMeta}}};

use ydb_unofficial::generated::ydb::{table::{ExecuteDataQueryRequest, query::Query, self, TransactionControl, TransactionSettings, transaction_settings::TxMode, transaction_control::TxSelector}, discovery::ListEndpointsRequest};


#[tokio::main]
pub async fn main() {
    init_logger();
    let url = "grpcs://ydb.serverless.yandexcloud.net:2135";
    let db_name = "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f";
    //let url = "grpcs://localhost:2135";
    //let db_name = "/local";
    let creds = env::var("TOKEN").unwrap();
    let uri: Uri = url.try_into().unwrap();
    let ep = ydb_unofficial::client::create_endpoint(url.try_into().unwrap());
    let channel = ep.connect().await.unwrap();
    let pool = YdbPoolBuilder::new(creds, db_name.try_into().unwrap(), uri.try_into().unwrap()).build().unwrap();
    let f1 = create_table2(&pool, db_name);
    let f2 = create_table3(&pool, db_name);
    let res = tokio::try_join!(f1, f2).unwrap();
    if false {
        let mut service = pool.get().await.unwrap();

        //client::Client::new(url, db_name, creds.to_owned()).await.unwrap();
        let mut discovery = pool.get().await.unwrap();
        let mut discovery = discovery.discovery();
        //let mut client = DiscoveryServiceClient::connect("test").await.unwrap();
        let response = discovery.list_endpoints(ListEndpointsRequest{database: db_name.into(), ..Default::default()}).await.unwrap();
        let payload = response.into_inner().result().unwrap();
        log::info!("payload: {payload:?}\n");

    //let mut table_client = TableServiceClient::connect("").await.unwrap();
        let query = "SELECT 1+1 as sum, 2*2 as mul";
        let session = service.table().await.unwrap();
        let mut transaction = ydb_unofficial::client::YdbTransaction::create(session).await.unwrap();
        let x = transaction.execute_data_query(ExecuteDataQueryRequest{
            query: Some(table::Query{query: Some(Query::YqlText(query.into()))}),
            ..Default::default()
        }).await.unwrap();

        let payload = x.into_inner().result();
        log::info!("payload: {:?}", payload);
        
        let x = transaction.execute_data_query(ExecuteDataQueryRequest{
            query: Some(table::Query{query: Some(Query::YqlText(query.into()))}),
            ..Default::default()
        }).await.unwrap();

        log::info!("\nx: {x:?}");
        let payload = x.into_inner().result();
        log::info!("\npayload: {payload:?}");


        let (mut session, _) = transaction.commit().await;
        //session.query("SELECT 1+1 as sum, 2*2 as mul".into()).await.unwrap();
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
    pool.close();
    tokio::time::sleep(Duration::from_secs(1)).await;
    
}
async fn create_table2(pool: &deadpool::managed::Pool<ConnectionManager<String>>, db_name: &str) -> Result<(), Box<dyn Error>> {
    let mut conn = pool.get().await?;
    let mut conn = conn.table().await?;
    let response = conn.execute_scheme_query(ExecuteSchemeQueryRequest {
        yql_text: "create table my_table2(id uint64 not null, value utf8, primary key(id))".to_owned(),
        ..Default::default()
    }).await?;
    log::error!("response: {response:?}");
    Ok(())
}
async fn create_table3(pool: &deadpool::managed::Pool<ConnectionManager<String>>, db_name: &str) -> Result<(), Box<dyn Error>> {
    let mut conn = pool.get().await?;
    let mut conn = conn.table().await?;
    let response = conn.execute_scheme_query(ExecuteSchemeQueryRequest {
        yql_text: "create table my_table3(id uint64 not null, value utf8, primary key(id))".to_owned(),
        ..Default::default()
    }).await?;
    log::error!("response: {response:?}");
    Ok(())
}
async fn create_table(pool: &deadpool::managed::Pool<ConnectionManager<String>>, db_name: &str) -> Result<(), Box<dyn Error>> {
    let str_type = ydb_unofficial::generated::ydb::Type {r#type: Some(ydb_unofficial::generated::ydb::r#type::Type::TypeId(PrimitiveTypeId::Utf8 as i32))};
    let str_nullable_type = ydb_unofficial::generated::ydb::Type {r#type: Some(ydb_unofficial::generated::ydb::r#type::Type::OptionalType(
        Box::new(ydb_unofficial::generated::ydb::OptionalType {item: Some(Box::new(
            ydb_unofficial::generated::ydb::Type {r#type: Some(ydb_unofficial::generated::ydb::r#type::Type::TypeId(PrimitiveTypeId::Utf8 as i32))}
        ))})
    ))};
    let req = CreateTableRequest{ 
        path: format!("{db_name}/my_table"),
        columns: vec![
            ColumnMeta { name: "id".to_owned(), r#type: Some(str_type.clone()), family: "".to_owned() },
            ColumnMeta { name: "value1".to_owned(), r#type: Some(str_nullable_type), family: "".to_owned() }
        ], 
        primary_key: vec!["id".to_owned()], 
        indexes: vec![], 
        ..Default::default()
    };
    let mut conn = pool.get().await?;
    let mut conn = conn.table().await?;
    let result = conn.create_table(req).await?;
    log::error!("result of create table: {result:?}");
    Ok(())
}

fn init_logger() {
    use simplelog::*;
    let mut builder = ConfigBuilder::new();
    builder.set_time_level(LevelFilter::Error);
    TermLogger::init(LevelFilter::Info, builder.build(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();
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


