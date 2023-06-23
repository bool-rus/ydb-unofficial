#![allow(dead_code)]
use std::error::Error;
use std::{env, time::Duration};

use tokio::sync::futures;
use tonic::codegen::CompressionEncoding;
use crate::client::YdbConnection;
use crate::generated::ydb::table::{ExecuteScanQueryRequest, ExecuteSchemeQueryRequest};
//use ydb_grpc::ydb_proto::{discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, ListEndpointsRequest}, table::{v1::table_service_client::TableServiceClient, CreateSessionRequest}};
use crate::{YdbResponseWithResult, generated::ydb::r#type::PrimitiveTypeId};
use crate::pool::YdbPoolBuilder;
use tonic::transport::Uri;
use crate::{pool::ConnectionManager, YdbError, generated::ydb::{table::{CreateTableRequest, ColumnMeta}}};

use crate::generated::ydb::{table::{ExecuteDataQueryRequest, query::Query, self, TransactionControl, TransactionSettings, transaction_settings::TxMode, transaction_control::TxSelector}, discovery::ListEndpointsRequest};


#[tokio::test]
pub async fn test() {
    init_logger();
    let url = "grpcs://ydb.serverless.yandexcloud.net:2135";
    let db_name = "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f";
    //let url = "grpcs://localhost:2135";
    //let db_name = "/local";
    let creds = env::var("DB_TOKEN").unwrap();
    let uri: Uri = url.try_into().unwrap();
    let ep = crate::client::create_endpoint(url.try_into().unwrap());
    let channel = ep.connect().await.unwrap();
    log::info!("channel connected");
    let mut service = YdbConnection::new(channel, db_name.try_into().unwrap(), creds.clone());
    log::info!("ydb connection created");
    let t = service.table().await.unwrap();
    log::info!("table client created");
    let pool = YdbPoolBuilder::new(creds, db_name.try_into().unwrap(), uri.try_into().unwrap())
        //.wait_timeout(Some(Duration::from_secs(5)))
        //.create_timeout(Some(Duration::from_secs(5)))
        //.recycle_timeout(Some(Duration::from_secs(5)))
        .build().unwrap();
    log::info!("pool created");
    let f1 = create_table2(&pool, db_name);
    let f2 = create_table3(&pool, db_name);
    let res = tokio::try_join!(f1, f2).unwrap();
    log::info!("tables created");
    if true {
        let mut service = pool.get().await.unwrap();

        //client::Client::new(url, db_name, creds.to_owned()).await.unwrap();
        let mut discovery = pool.get().await.unwrap();
        let mut discovery = discovery.discovery();
        //let mut client = DiscoveryServiceClient::connect("test").await.unwrap();
        let response = discovery.list_endpoints(ListEndpointsRequest{database: db_name.into(), ..Default::default()}).await.unwrap();
        let payload = response.get_ref().result().unwrap();
        let payload: ydb_grpc_bindings::generated::ydb::discovery::ListEndpointsResult = response.into_inner().result().unwrap();
        log::info!("payload: {payload:?}\n");

    //let mut table_client = TableServiceClient::connect("").await.unwrap();
        let query = "SELECT 1+1 as sum, 2*2 as mul";
        let session = service.table().await.unwrap();
        let mut transaction = crate::client::YdbTransaction::create(session).await.unwrap();
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
        let payload = x.into_inner().result().unwrap();
        log::info!("\npayload: {payload:?}");
        let rs = payload.result_sets.first().unwrap();
        log::info!("rs: {rs:?}");
        for r in &rs.rows {
            log::info!("row: {r:?}");
            for i in &r.items {
                log::info!("item: {i:?}");
            }
        }
        


        let (mut session, _) = transaction.commit().await;
        //session.query("SELECT 1+1 as sum, 2*2 as mul".into()).await.unwrap();
    }
    tokio::time::sleep(Duration::from_secs(3)).await;
    pool.close();
    tokio::time::sleep(Duration::from_secs(1)).await;
    
}
async fn create_table2(pool: &deadpool::managed::Pool<ConnectionManager<String>>, db_name: &str) -> Result<(), Box<dyn Error>> {
    log::info!("create table 2 started");
    let mut conn = pool.get().await?;
    log::info!("2: conn invoked");
    let mut conn = conn.table().await?;
    let response = conn.execute_scheme_query(ExecuteSchemeQueryRequest {
        yql_text: "create table my_table2(id uint64 not null, value utf8, primary key(id))".to_owned(),
        ..Default::default()
    }).await?;
    log::info!("response: {response:?}");
    Ok(())
}
async fn create_table3(pool: &deadpool::managed::Pool<ConnectionManager<String>>, db_name: &str) -> Result<(), Box<dyn Error>> {
    log::info!("create table 3 started");
    let mut conn = pool.get().await?;
    log::info!("3: conn invoked");
    let mut conn = conn.table().await?;
    let response = conn.execute_scheme_query(ExecuteSchemeQueryRequest {
        yql_text: "create table my_table3(id uint64 not null, value utf8, primary key(id))".to_owned(),
        ..Default::default()
    }).await?;
    log::info!("response: {response:?}");
    Ok(())
}
async fn create_table(pool: &deadpool::managed::Pool<ConnectionManager<String>>, db_name: &str) -> Result<(), Box<dyn Error>> {
    let str_type = crate::generated::ydb::Type {r#type: Some(crate::generated::ydb::r#type::Type::TypeId(PrimitiveTypeId::Utf8 as i32))};
    let str_nullable_type = crate::generated::ydb::Type {r#type: Some(crate::generated::ydb::r#type::Type::OptionalType(
        Box::new(crate::generated::ydb::OptionalType {item: Some(Box::new(
            crate::generated::ydb::Type {r#type: Some(crate::generated::ydb::r#type::Type::TypeId(PrimitiveTypeId::Utf8 as i32))}
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

