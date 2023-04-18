use ydb_grpc::ydb_proto::discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, ListEndpointsRequest};
use exper::YdbResponse;

mod pool;
mod client;
mod exper;

#[tokio::main]
pub async fn main() {
    println!("hello world");
    let url = "grpcs://ydb.serverless.yandexcloud.net:2135";
    let db_name = "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f";
    let url = "grpc://localhost:2136";
    let db_name = "/local";
    let creds = "";
    let ep = client::create_endpoint(url.try_into().unwrap());
    let channel = ep.connect().await.unwrap();
    let service = client::create_ydb_service(channel, db_name.into(), creds.to_owned());

    //client::Client::new(url, db_name, creds.to_owned()).await.unwrap();

    let mut client = DiscoveryServiceClient::new(service);
    let response = client.list_endpoints(ListEndpointsRequest{database: db_name.into(), ..Default::default()}).await.unwrap();
    let payload = response.into_inner().payload().unwrap();
    println!("payload: {payload:?}");


}

