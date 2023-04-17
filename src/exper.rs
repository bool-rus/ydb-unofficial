use tonic::{transport::Channel, service::Interceptor};
use tower::ServiceBuilder;
use ydb_grpc::ydb_proto::discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIResponse, WhoAmIResult, ListEndpointsResponse, ListEndpointsResult};




async fn new_service() {
    let channel = Channel::from_static("ya.ru").connect().await.unwrap();
    let db_name = "bgg".try_into().unwrap();
    let creds = "bgg".to_owned();
    let interceptor = DBInterceptor {db_name, creds};
    let channel = ServiceBuilder::new()
        // Interceptors can be also be applied as middleware
        .layer(tonic::service::interceptor(interceptor))
       //.layer_fn(AuthSvc::new)
        .service(channel);
    let mut client = DiscoveryServiceClient::new(channel);

}

fn add_auth(channel: Channel, db_name: String, creds: String) -> impl tonic::client::GrpcService<tonic::body::BoxBody> {
    vec![1];
    let db_name = "bgg".try_into().unwrap();
    let creds = "bgg".to_owned();
    let interceptor = DBInterceptor {db_name, creds};
    ServiceBuilder::new()
        // Interceptors can be also be applied as middleware
        .layer(tonic::service::interceptor(interceptor))
        .layer_fn(|x|x)
        .service(channel)
}

#[derive(Clone, Debug)]
pub struct DBInterceptor {
    db_name: String,
    creds: String,
}


impl Interceptor for DBInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let headers = request.metadata_mut();
        headers.insert("x-ydb-database", self.db_name.as_str().try_into().unwrap());
        headers.insert("x-ydb-sdk-build-info", "bgg".try_into().unwrap());
        headers.insert("x-ydb-auth-ticket", self.creds.as_str().try_into().unwrap());
        Ok(request)    
    }
}


pub trait YdbResponse {
    type Payload;
    fn payload(&self) -> Option<Self::Payload>;
}


macro_rules! payloaded {
    ($x:ty , $p:ty) => {
        impl YdbResponse for $x {
            type Payload = $p;
            fn payload(&self) -> Option<Self::Payload> {
                use prost::Message;
                let bytes = self.operation.as_ref()?.result.as_ref()?.value.as_slice();
                Some(Message::decode(bytes).unwrap())
            }
        }
    }
}


payloaded!(WhoAmIResponse , WhoAmIResult);
payloaded!(ListEndpointsResponse , ListEndpointsResult);