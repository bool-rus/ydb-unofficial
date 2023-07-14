
use crate::generated::ydb::{table, discovery};
use table::*;
use discovery::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractResultError {
    #[error("Empty body of response")]
    Empty,
    #[error("Cannot decode result: {0}")]
    Decode(#[from] prost::DecodeError),
}

/// The trait to invoke payload result from response. See examples in [`crate::client`]
pub trait YdbResponseWithResult {
    type Result;
    fn result(&self) -> Result<Self::Result,ExtractResultError>;
}


macro_rules! payloaded {
    ($($x:ty : $p:ty,)+) => {$(
        impl YdbResponseWithResult for $x {
            type Result = $p;
            fn result(&self) -> Result<Self::Result, ExtractResultError> {
                use prost::Message;
                use ExtractResultError::*;
                let operation = self.operation.as_ref().ok_or(Empty)?;
                let bytes = operation
                    .result.as_ref().ok_or(Empty)?
                    .value.as_slice();
                Message::decode(bytes).map_err(|e|Decode(e))
            }
        }
    )+}
}

pub(crate) use payloaded;

payloaded!(
    WhoAmIResponse: WhoAmIResult, 
    ListEndpointsResponse: ListEndpointsResult,
    CreateSessionResponse: CreateSessionResult,
    ExecuteDataQueryResponse: ExecuteQueryResult,
    BeginTransactionResponse: BeginTransactionResult,
    CommitTransactionResponse: CommitTransactionResult,
);