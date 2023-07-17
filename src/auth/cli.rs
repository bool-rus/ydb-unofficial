use std::sync::{RwLock, Arc};
use std::time::Duration;
use crate::AsciiValue;
use super::Credentials;

#[derive(Debug, Clone)]
/// An automaitc updatable token.
/// Updates every 11 hours by run command `yc iam create-token`.
/// To use that you need [Yandex Cloud CLI](https://cloud.yandex.ru/docs/cli/operations/install-cli) installed
pub struct Cli {
    token: Arc<RwLock<AsciiValue>>,
    abort_handle: Arc<tokio::task::AbortHandle>,
}

impl Credentials for Cli {
    fn token(&self) -> AsciiValue {
        self.token.read().unwrap().clone()
    }
}

impl Cli {
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

impl Drop for Cli {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}