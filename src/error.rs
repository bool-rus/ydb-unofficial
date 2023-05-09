use thiserror::Error;

use crate::ExtractResultError;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum YdbError {
    Grpc(#[from] tonic::Status),
    ExtractResultError(#[from] ExtractResultError)

}
