use std::sync::{Arc, RwLock};
use std::time::{UNIX_EPOCH, SystemTime, Duration};

use jwt_simple::prelude::{Claims, RSAKeyPairLike};
pub use jwt_simple::prelude::PS256KeyPair;
use serde::Deserialize;
use tonic::transport::Uri;
use yandex_cloud::yandex::cloud::iam::v1::CreateIamTokenResponse;

use crate::AsciiValue;
use super::{Credentials, UpdatableToken};

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    pub id: String,
    pub service_account_id: String,
    #[serde(with="ps256_private_key")]
    pub private_key: PS256KeyPair,
}

#[derive(Debug, Clone)]
pub struct UpdateConfig {
    // Auth endpoint. Default is grpcs://iam.api.cloud.yandex.net:443
    pub endpoint: Uri,
    /// One of JWT cliams. Default is https://iam.api.cloud.yandex.net/iam/v1/tokens
    pub audience: String,
    /// Default update period. Used if received auth response without `expired_at`. Default is 50 minutes.
    pub update_period: Duration,
    /// Time reserve to update token. Default is 1 minute.
    pub update_time_reserve: Duration,
    /// JWT claim expiration. Default is 1 minute.
    pub token_request_claim_time: Duration,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self { 
            endpoint: "grpcs://iam.api.cloud.yandex.net:443".parse().unwrap(), 
            audience: "https://iam.api.cloud.yandex.net/iam/v1/tokens".into(), 
            update_period: Duration::from_secs(50*60), 
            update_time_reserve: Duration::from_secs(60),
            token_request_claim_time: Duration::from_secs(60),
        }
    }
}

#[derive(Clone)]
pub struct ServiceAccountCredentials {
    token: Arc<RwLock<AsciiValue>>,
}

impl Credentials for ServiceAccountCredentials {
    fn token(&self) -> crate::AsciiValue {
        self.token.read().unwrap().clone()
    }
}
impl Into<UpdatableToken> for ServiceAccountCredentials {
    fn into(self) -> UpdatableToken {
        let Self{token} = self;
        UpdatableToken { token }
    }
}

impl ServiceAccountCredentials {
    pub async fn create(key: ServiceAccountKey) -> Result<Self, tonic::Status> {
        Self::create_with_config(Default::default(), key).await
    }
    pub async fn create_with_config(conf: UpdateConfig, key: ServiceAccountKey) -> Result<Self, tonic::Status> {
        //TODO: переделать на Stream
        let response = conf.request_iam_token(&key).await?;
        let mut sleep_duration = conf.invoke_sleep_duration(&response);
        let token = Arc::new(RwLock::new(response.iam_token.clone().try_into().unwrap()));
        let update_me = Arc::downgrade(&token);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(sleep_duration).await;
                if let Some(token) = update_me.upgrade() {
                    match conf.request_iam_token(&key).await {
                        Ok(response) => {
                            sleep_duration = conf.invoke_sleep_duration(&response);
                            *token.write().unwrap() = response.iam_token.clone().try_into().unwrap();
                            log::info!("Iam token updated");
                        }
                        Err(e) => {
                            log::error!("Cannot update iam token: {:?}", e);
                            sleep_duration = Duration::from_secs(5);
                        }
                    }
                } else {
                    log::info!("ServiceAccountCredentials removed");
                    break;
                }
            }
        });
        Ok(Self {token})
    }
}


impl UpdateConfig {
    pub async fn request_iam_token(&self, key: &ServiceAccountKey) -> Result<CreateIamTokenResponse, tonic::Status> {
        let jwt = self.make_jwt(key);
        let endpoint = crate::client::create_endpoint(self.endpoint.clone());
        let mut client = yandex_cloud::yandex::cloud::iam::v1::iam_token_service_client::IamTokenServiceClient::new(endpoint.connect_lazy());
        let request = yandex_cloud::yandex::cloud::iam::v1::CreateIamTokenRequest {
            identity: Some(yandex_cloud::yandex::cloud::iam::v1::create_iam_token_request::Identity::Jwt(jwt))
        };
        let resp = client.create(request).await?;
        Ok(resp.into_inner())
    }

    pub fn make_jwt(&self, key: &ServiceAccountKey) -> String {
        let claims = Claims::create(self.token_request_claim_time.into())
        .with_issuer(&key.service_account_id)
        .with_audience(&self.audience);
        let pair = key.private_key.clone().with_key_id(&key.id);
        let token = pair.sign(claims).unwrap();
        token
    }
    pub fn invoke_sleep_duration(&self, response: &CreateIamTokenResponse) -> tokio::time::Duration {
        let CreateIamTokenResponse {iam_token: _, expires_at} = response;
        let expires = if let Some(ts) = expires_at {
            (UNIX_EPOCH + Duration::from_secs(ts.seconds as u64)) //convert ts to SystemTime
            .duration_since(SystemTime::now() + self.update_time_reserve) //расчитываем время сна (с запасом)
            .unwrap_or_default()
        } else  {
            self.update_period
        };
        Duration::from_secs(60)//expires.into();
    }
}

mod ps256_private_key {
    use jwt_simple::prelude::PS256KeyPair;
    use serde::{self, Deserialize, Deserializer};
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<PS256KeyPair, D::Error>
    where
    D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PS256KeyPair::from_pem(&s).map_err(serde::de::Error::custom)
    }
}
