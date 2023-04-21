use tonic::{transport::Channel, service::Interceptor};
use tower::ServiceBuilder;

use crate::generated::ydb::{discovery::{WhoAmIResponse, WhoAmIResult, ListEndpointsResponse, ListEndpointsResult}, table::{CreateSessionResponse, CreateSessionResult, DeleteSessionResponse, ExecuteDataQueryResponse, ExecuteQueryResult}, status_ids::StatusCode};
//use ydb_grpc::ydb_proto::{discovery::{v1::discovery_service_client::DiscoveryServiceClient, WhoAmIResponse, WhoAmIResult, ListEndpointsResponse, ListEndpointsResult}, table::{CreateSessionResponse, CreateSessionResult}};

#[derive(Debug)]
pub enum ExtractPayloadError {
    Empty,
    BadSession,
    SessionBusy,
    Parse,
    Unknown,
}

impl std::fmt::Display for ExtractPayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}
impl std::error::Error for ExtractPayloadError {}


pub trait YdbResponse {
    type Payload;
    fn payload(&self) -> Result<Self::Payload,ExtractPayloadError>;
}


macro_rules! payloaded {
    ($x:ty , $p:ty) => {
        impl YdbResponse for $x {
            type Payload = $p;
            fn payload(&self) -> Result<Self::Payload, ExtractPayloadError> {
                use prost::Message;
                use ExtractPayloadError::*;
                let operation = self.operation.as_ref().ok_or(Empty)?;
                match operation.status() {
                    StatusCode::Success => Ok(()),
                    StatusCode::BadSession => Err(BadSession),
                    StatusCode::SessionExpired => Err(BadSession),
                    StatusCode::SessionBusy => Err(SessionBusy),
                    _ => Err(Unknown),
                }?;
                let bytes = operation
                    .result.as_ref().ok_or(Empty)?
                    .value.as_slice();
                Message::decode(bytes).map_err(|_|Parse)
            }
        }
    }
}


payloaded!(WhoAmIResponse , WhoAmIResult);
payloaded!(ListEndpointsResponse , ListEndpointsResult);
payloaded!(CreateSessionResponse, CreateSessionResult);
payloaded!(ExecuteDataQueryResponse, ExecuteQueryResult);