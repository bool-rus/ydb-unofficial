use std::{env::var, error::Error};

use tokio::fs::File;
use ydb_grpc_bindings::generated::ydb::table::{PrepareDataQueryRequest, ExplainDataQueryRequest, TransactionControl, transaction_control::TxSelector, TransactionSettings, transaction_settings::TxMode, ExecuteDataQueryRequest};

use crate::{auth::sa::ServiceAccountKey, YdbConnection, client::create_endpoint, YdbResponseWithResult};

#[tokio::test]
async fn explain() -> Result<(), Box<dyn Error>> {
    let key: ServiceAccountKey = serde_json::from_reader(
        std::fs::File::open("test-env/authorized_key.json")?
    ).unwrap();
    let creds = crate::auth::sa::ServiceAccountCredentials::create(key).await?;
    let url = var("YDB_URL").expect("YDB_URL not set");
    let db_name = var("DB_NAME").expect("DB_NAME not set");
    let endpoint = create_endpoint(url.try_into()?);
    let channel = endpoint.connect_lazy();
    let mut conn = YdbConnection::new(channel, db_name.as_str().try_into()?, creds);
    let mut table = conn.table().await.unwrap();


    println!("=======================");
    let response = table.explain_data_query(ExplainDataQueryRequest { 
        yql_text: "insert into bot_admins values (?1, ?2, ?3)".into(), ..Default::default() 
    }).await?;
    println!("\nexplain issues: {response:?}");
    let result = response.into_inner().result()?;
    println!("\nquery ast: {}", result.query_ast);
    println!("\nquery plan: {}", result.query_plan);
    Ok(())
}
#[tokio::test]
async fn select() -> Result<(), Box<dyn Error>> {

    let key: ServiceAccountKey = serde_json::from_reader(
        std::fs::File::open("test-env/authorized_key.json")?
    ).unwrap();
    let creds = crate::auth::sa::ServiceAccountCredentials::create(key).await?;
    let url = var("YDB_URL").expect("YDB_URL not set");
    let db_name = var("DB_NAME").expect("DB_NAME not set");
    let endpoint = create_endpoint(url.try_into()?);
    let channel = endpoint.connect_lazy();
    let mut conn = YdbConnection::new(channel, db_name.as_str().try_into()?, creds);
    let mut table = conn.table().await.unwrap();

    let yql = "select * from test_decl;".to_owned();
    let query = Some(crate::generated::ydb::table::query::Query::YqlText(yql));
    let query = Some(crate::generated::ydb::table::Query{query});

    let tx_control = Some(TransactionControl { 
        commit_tx: true, 
        tx_selector: Some(TxSelector::BeginTx(TransactionSettings { 
            tx_mode: Some(TxMode::SerializableReadWrite(Default::default())) 
        })) 
    });
    let response = table.execute_data_query(ExecuteDataQueryRequest{query, tx_control, collect_stats: 2, ..Default::default()}).await?;
    tokio::fs::write("test/example.protobytes", response.get_ref().operation.as_ref().unwrap().result.as_ref().unwrap().value.as_slice()).await?;
    let result = response.get_ref().result()?;
    println!("result: {result:?}");
    for rs in &result.result_sets {
        println!("\n\n new result set ===========");
        println!("======columns: ");
        for col in &rs.columns {
            
            println!("{col:?}");
        }
        println!("\n======rows:");
        for r in &rs.rows {
            let r: Vec<_> = r.items.iter().map(|v|(&v.high_128, &v.value)).collect();
            println!("{r:?}");
        }
    }
    println!("u64 max: {}", u64::MAX - 2*1_000_000_000 + 1);
    Ok(())

}