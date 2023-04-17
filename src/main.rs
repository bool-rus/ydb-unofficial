use ydb_grpc::ydb_proto::discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest};
use exper::YdbResponse;

mod pool;
mod client;
mod exper;

#[tokio::main]
pub async fn main() {
    println!("hello world");
    let ep = client::create_endpoint("grpcs://ydb.serverless.yandexcloud.net:2135".try_into().unwrap());
    let channel = ep.connect().await.unwrap();
    let creds = "token".to_owned();
    let service = client::create_ydb_service(channel, "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f".into(), creds);

    let mut client = DiscoveryServiceClient::new(service);
    let response = client.who_am_i(WhoAmIRequest::default()).await.unwrap();
    let iam = response.into_inner().payload().unwrap();
    iam.user;
    iam.groups;

    let client = client::Client::new(
        "grpcs://ydb.serverless.yandexcloud.net:2135/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f", 
        "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f",
        "t1.9euelZqUzcuOmZnMks-MxpKYjI-Km-3rnpWamcmNip2Tk46RxpyZlpuTyo_l8_d6URBe-e94Ek81_d3z9zoADl7573gSTzX9.ca2UcJS5Vnjqe7EvvV45C5mF0xQxyXXfOaUodSQtKJitDMMA4zuW7HdLFmPhX1GSp15ZSXmKC5WdWZqknf3DBw".to_owned(), 
    ).await.unwrap();
    println!("client: {client:?}");
    //let ami= client.whoami().await.unwrap();
    //println!("i am: {ami:?}");
}

