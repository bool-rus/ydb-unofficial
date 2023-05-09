
use crate::generated::ydb::{table, discovery, status_ids};
use table::*;
use discovery::*;
use status_ids::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractResultError {
    #[error("Empty body of response")]
    Empty,
    #[error("Bad session")] 
    BadSession, //TODO: наверное, стоит эту ошибку вообще во всех методах для табличного сервиса использовать
    #[error("Session busy")] 
    SessionBusy,
    #[error("Cannot parse result")]
    Parse,
    #[error("Unknown operation status")]
    Unknown,
}


pub trait YdbResponseWithResult {
    type Payload;
    fn result(&self) -> Result<Self::Payload,ExtractResultError>;
}


macro_rules! payloaded {
    ($($x:ty : $p:ty,)+) => {$(
        impl YdbResponseWithResult for $x {
            type Payload = $p;
            fn result(&self) -> Result<Self::Payload, ExtractResultError> {
                use prost::Message;
                use ExtractResultError::*;
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
    )+}
}


payloaded!(
    WhoAmIResponse: WhoAmIResult, 
    ListEndpointsResponse: ListEndpointsResult,
    CreateSessionResponse: CreateSessionResult,
    ExecuteDataQueryResponse: ExecuteQueryResult,
    BeginTransactionResponse: BeginTransactionResult,
    CommitTransactionResponse: CommitTransactionResult,
);