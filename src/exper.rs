use tonic::{transport::Channel, service::Interceptor};
use tower::ServiceBuilder;

use crate::generated::ydb::{discovery::{WhoAmIResponse, WhoAmIResult, ListEndpointsResponse, ListEndpointsResult}, table::{CreateSessionResponse, CreateSessionResult}};
//use ydb_grpc::ydb_proto::{discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIResponse, WhoAmIResult, ListEndpointsResponse, ListEndpointsResult}, table::{CreateSessionResponse, CreateSessionResult}};








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
payloaded!(CreateSessionResponse, CreateSessionResult);