use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;
use deadpool::managed;
use tonic::transport::{Channel, Endpoint};
use ydb_grpc::ydb_proto::discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest};

#[derive(Debug)]
struct Manager {}

#[async_trait]
impl managed::Manager for Manager {
    type Type = Channel;
    type Error = tonic::transport::Error;
    
    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let e: Endpoint = "https://ya.ru/".try_into().unwrap();
        e.connect().await
    }
    
    async fn recycle(&self, _: &mut Self::Type) -> managed::RecycleResult<Self::Error> {
        println!("recycle");
        Ok(())
    }
}

type Pool = managed::Pool<Manager>;

#[tokio::test]
async fn my_test() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).build().unwrap();
    let pool1 = pool.clone();
    tokio::spawn(async {
        go_with_pool(pool1).await;
    });
    let pool1 = pool.clone();
    tokio::spawn(async {
        go_with_pool(pool1).await;
    });
    
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    println!("pool: {:?}", pool.status());
}
#[cfg(test)]
async fn go_with_pool(pool: Pool) {
    use std::{time::Duration, sync::Arc};

    let mut conn = pool.get().await.unwrap();
    let mut client = DiscoveryServiceClient::new(conn.clone());
    tokio::time::sleep(Duration::from_secs(1)).await;
    let x = client.who_am_i(WhoAmIRequest::default()).await;
    println!("x: {}", x.is_ok());
}
