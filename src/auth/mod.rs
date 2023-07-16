//! Staff to implement authentication to Ydb.
//! You can make your own auth by implement [`Credentials`]
use std::{sync::{RwLock, Arc}, time::Duration};

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
/// An automaitc updatable token.
/// Updates every 11 hours by run command `yc iam create-token`.
/// To use that you need [Yandex Cloud CLI](https://cloud.yandex.ru/docs/cli/operations/install-cli) installed
pub struct YcEnv {
    token: Arc<RwLock<AsciiValue>>,
    abort_handle: Arc<tokio::task::AbortHandle>,
}

impl Credentials for YcEnv {
    fn token(&self) -> AsciiValue {
        self.token.read().unwrap().clone()
    }
}

impl YcEnv {
    pub async fn new() -> Self {
        let token = Self::create_token().await;
        let token = Arc::new(RwLock::new(token));
        let update_me = token.clone();
        let abort_handle = tokio::spawn(async move {
            let mut timer = tokio::time::interval(Duration::from_secs(60*60*11));
            loop {
                timer.tick().await;
                let token = Self::create_token().await;
                *update_me.write().unwrap() = token;
            }
        }).abort_handle();
        let abort_handle = Arc::new(abort_handle);
        Self {token, abort_handle}
    }
    async fn create_token() -> AsciiValue {
        let out = tokio::process::Command::new("yc").arg("iam").arg("create-token").output().await.expect("cannot run `yc iam create-token`");
        let stdout = out.stdout.as_slice();
        let stdout = &stdout[0..stdout.len() - 1];
        AsciiValue::try_from(stdout).unwrap()
    }
}

impl Drop for YcEnv {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}

#[cfg(feature = "auth-sa")]
/// Service account authentication implementation. Uses authorized key (in json) created by Yandex Cloud
/// Implements [`Credentials`] with auto-updatable token
/// 
/// # Examples
/// 
/// ``` rust
/// # #[tokio::main]
/// # async fn main() {
/// use ydb_unofficial::auth::service_account::{ServiceAccountKey, ServiceAccountCredentials};
/// let path = "test-env/authorized_key.json";
/// let file = std::fs::File::open(path).unwrap();
/// let key: ServiceAccountKey = serde_json::from_reader(file).unwrap();
/// let creds = ServiceAccountCredentials::create(key).await.unwrap();
/// # }
/// ```
pub mod service_account;