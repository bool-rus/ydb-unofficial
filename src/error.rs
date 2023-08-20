//! Some error types that may appear when interacting with Ydb
use std::fmt::Display;

use thiserror::Error;

use crate::generated::ydb::operations::Operation;
pub use crate::payload::ExtractResultError;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum YdbError {
    Grpc(#[from] tonic::Status),
    ExtractResultError(#[from] ExtractResultError),
    #[error("Error from ydb: {0}")]
    Ydb(ErrWithOperation),
    #[error("Empty response")]
    EmptyResponse,
    #[cfg(feature = "sqlx")]
    #[error("Error on decode ast")]
    DecodeAst,
}

#[derive(Error, Debug)]
pub struct ErrWithOperation(pub Operation);

impl Display for ErrWithOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = self.0.status();
        write!(f, "Operation status: {status:?}")
    }
}