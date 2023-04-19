use tonic::{transport::{Certificate, ClientTlsConfig, Channel}, codegen::InterceptedService};
//use ydb_grpc::ydb_proto::{discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIRequest, ListEndpointsRequest}, table::{v1::table_service_client::TableServiceClient, CreateSessionRequest}};
use exper::YdbResponse;
use generated::google::protobuf::Any;

use crate::generated::{ydb::{discovery::{ListEndpointsRequest, ListEndpointsResponse, v1::MyStruct}, table::{v1::table_service_client::TableServiceClient, CreateSessionRequest}}, DiscoveryServiceClient};

mod pool;
mod client;
mod exper;
mod generated;


#[ctor::ctor]
static CERT: Certificate = {
    //let cert="MIIB0zCCATygAwIBAgICA+gwDQYJKoZIhvcNAQELBQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTIzMDQxODA1NDIyOVoXDTMzMDQxNTA1NDIyOVowFDESMBAGA1UEAwwJbG9jYWxob3N0MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQCzlr16HVGIrWDyNKJ9ckbl+MNyNY94s5UD8OPonkPsDIhKcIHJDG5qMrcxMeEf/YBhCPYO0/OftEdrEr3lU092ecg6EtFAl0j27die29+Z62op+Iw9bMTkwuUfOBka0sLhux93ZtZ5ODBdtinnV0z6KUPUhwEKQ8Rxn+E0M1nXrQIDAQABozQwMjAPBgNVHRMECDAGAQH/AgEAMB8GA1UdEQQYMBaCCWxvY2FsaG9zdIIJbG9jYWxob3N0MA0GCSqGSIb3DQEBCwUAA4GBAH4yFUD2vSA1AXxnfkqg3LwlyUjzsKE3o109Xn0A08WgnhL87ksrHoaTAKPxY+ONiZmp2fbL7+TxH6wWbCAxi0GnEof89ElZfvJJyK9sD+cEMIoFh6/zb6dG13EHksmSNUnhXjdK3i4yOmdedS497rOzxA26PZ7bSSWPb6TFLJj5";
    let cert = "MIICgDCCAemgAwIBAgIUNIQJpAG6AD2Nq4AU1reVG0pRGdAwDQYJKoZIhvcNAQELBQAwUjELMAkGA1UEBhMCQVUxEzARBgNVBAgMClNvbWUtU3RhdGUxITAfBgNVBAoMGEludGVybmV0IFdpZGdpdHMgUHR5IEx0ZDELMAkGA1UEAwwCY2EwHhcNMjMwNDE4MDkxNDU4WhcNMzMwMTE1MDkxNDU4WjBSMQswCQYDVQQGEwJBVTETMBEGA1UECAwKU29tZS1TdGF0ZTEhMB8GA1UECgwYSW50ZXJuZXQgV2lkZ2l0cyBQdHkgTHRkMQswCQYDVQQDDAJjYTCBnzANBgkqhkiG9w0BAQEFAAOBjQAwgYkCgYEAs5a9eh1RiK1g8jSifXJG5fjDcjWPeLOVA/Dj6J5D7AyISnCByQxuajK3MTHhH/2AYQj2DtPzn7RHaxK95VNPdnnIOhLRQJdI9u3YntvfmetqKfiMPWzE5MLlHzgZGtLC4bsfd2bWeTgwXbYp51dM+ilD1IcBCkPEcZ/hNDNZ160CAwEAAaNTMFEwHQYDVR0OBBYEFOWpZhzDl/e+4h9eLVuNwzqPvPnLMB8GA1UdIwQYMBaAFOWpZhzDl/e+4h9eLVuNwzqPvPnLMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQELBQADgYEAhRFHSMmh9i+f6smZV/JfHIIwAbrR9/SY9SaYlyhUBs35OutcgCMl7DPdJgtCDGAyN1DFVxLeMn436suNhwERYFWBwdzqrDe5zBhZOueBRtgxqXks9loQG9h9ZTRr55PxADnB7iX4/Kpss4RXNxJpCcaPY9e7L8712PY2B4ssY+w=";
    let pem = base64::decode(cert).unwrap();
    Certificate::from_pem(pem)
};


#[tokio::main]
pub async fn main() {
    println!("hello world");
    let url = "grpcs://ydb.serverless.yandexcloud.net:2135";
    let db_name = "/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f";
    let url = "grpcs://localhost:2135";
    let db_name = "/local";
    let creds = "";
    let tls_config = ClientTlsConfig::new().ca_certificate(CERT.clone());
    //println!("tls config: {tls_config:?}");
    let ep = client::create_endpoint(url.try_into().unwrap()).tls_config(tls_config).unwrap();
    let channel = ep.connect().await.unwrap();
    let service = client::create_ydb_service(channel, db_name.into(), creds.to_owned());

    //client::Client::new(url, db_name, creds.to_owned()).await.unwrap();

    let mut client = DiscoveryServiceClient::new(service.clone());
    let mut client = DiscoveryServiceClient::connect("test").await.unwrap();
    //client.list_endpoints(request)
    //let mut client = DiscoveryServiceClient::connect(url).await.unwrap();
    let response = client.list_endpoints(ListEndpointsRequest{database: db_name.into(), ..Default::default()}).await.unwrap();
    let payload = response.into_inner().payload().unwrap();
    println!("payload: {payload:?}");

    let mut table_client = TableServiceClient::new(service.clone());
    let session_response = table_client.create_session(CreateSessionRequest::default()).await.unwrap();
    let result = session_response.into_inner().payload().unwrap();


}



trait Foo {type Inner;}
trait Bar {type Inner;}
impl Foo for i32 {type Inner = i32;}
impl Bar for i32 {type Inner = i32;}
struct Baz<T>(T);
impl<T> Baz<T> where T: Foo, T::Inner: Bar,
//<T::Inner as Bar>::Inner: Sized
{
    pub fn new(obj: T) -> Self {Self(obj)}
    pub fn foo(&self) -> String {"foobazz".to_owned()}
}

fn test() {
    let baz = Baz::new(1);
    let s = baz.foo();
}