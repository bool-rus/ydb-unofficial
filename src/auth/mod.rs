//! Staff to implement authentication to Ydb.
//! You can make your own auth by implement [`Credentials`]


use std::sync::{Arc, RwLock};

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

#[derive(Debug, Clone)]
pub struct UpdatableToken {
    token: Arc<RwLock<AsciiValue>>,
}

impl UpdatableToken {
    pub fn new(token: AsciiValue) -> Self {
        let token = Arc::new(RwLock::new(token));
        Self {token}
    }
}

impl Credentials for UpdatableToken {
    fn token(&self) -> AsciiValue {
        self.token.read().unwrap().clone()
    }
}

#[cfg(feature = "auth-cli")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth-cli")))]
/// [`Credentials`] implementation that create and updates token every 11 hours by run command `yc iam create-token`
pub mod cli;

#[cfg(feature = "auth-sa")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth-sa")))]
/// Service account authentication implementation. Uses authorized key (in json) created by Yandex Cloud
/// Implements [`Credentials`] with auto-updatable token
/// 
/// # Examples
/// 
/// ``` rust
/// # #[tokio::main]
/// # async fn main() {
/// use ydb_unofficial::auth::sa::{ServiceAccountKey, ServiceAccountCredentials};
/// let path = "test-env/authorized_key.json";
/// let file = std::fs::File::open(path).unwrap();
/// let key: ServiceAccountKey = serde_json::from_reader(file).unwrap();
/// let creds = ServiceAccountCredentials::create(key).await.unwrap();
/// # }
/// ```
pub mod sa;