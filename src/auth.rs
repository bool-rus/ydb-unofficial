//! Staff to implement authentication to Ydb.
//! You can make your own auth by implement [`Credentials`]
use super::*;
/// Trait to creates tokens for ydb auth
pub trait Credentials: Clone + Send + 'static {
    fn token(&self) -> AsciiValue;
}

impl Credentials for String {
    fn token(&self) -> AsciiValue {
        self.clone().try_into().unwrap()
    }
}