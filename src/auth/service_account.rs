

use std::default;
use std::sync::{Arc, RwLock};
use std::time::{UNIX_EPOCH, SystemTime, Duration};


use jwt_simple::prelude::{Claims, PS256KeyPair, PS256PublicKey, RSAKeyPairLike};
use serde::{Serialize, Deserialize};
use tonic::transport::Uri;
use yandex_cloud::yandex::cloud::iam::v1::CreateIamTokenResponse;

use crate::auth::Credentials;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountKey {
    pub id: String,
    pub service_account_id: String,
    pub created_at: String,
    pub key_algorithm: String,
    #[serde(with="ps256_public_key")]
    pub public_key: PS256PublicKey,
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
    token: Arc<RwLock<String>>,
}

impl Credentials for ServiceAccountCredentials {
    fn token(&self) -> crate::AsciiValue {
        self.token.read().unwrap().clone().try_into().unwrap()
    }
}

impl ServiceAccountCredentials {
    pub async fn create(key: ServiceAccountKey) -> Result<Self, tonic::Status> {
        Self::create_with_config(Default::default(), key).await
    }
    pub async fn create_with_config(conf: UpdateConfig, key: ServiceAccountKey) -> Result<Self, tonic::Status> {
        let mut response = conf.request_iam_token(&key).await?;
        let token = Arc::new(RwLock::new(response.iam_token.clone()));
        let result = Self {token};
        let token = Arc::downgrade(&result.token);
        tokio::spawn(async move {
            loop {
                let sleep_duration = conf.invoke_sleep_duration(&response);
                tokio::time::sleep(sleep_duration).await;
                response = conf.request_iam_token(&key).await.unwrap();
                if let Some(token) = token.upgrade() {
                    *token.write().unwrap() = response.iam_token.clone();
                    log::info!("Iam token updated");
                } else {
                    log::info!("ServiceAccountCredentials removed");
                    break;
                }
            }
        });
        Ok(result)
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
        expires.into()
    }
}

mod ps256_public_key {
    use jwt_simple::prelude::PS256PublicKey;
    use serde::{self, Deserialize, Serializer, Deserializer};
    pub fn serialize<S>(
        key: &PS256PublicKey,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = key.to_pem().map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<PS256PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PS256PublicKey::from_pem(&s).map_err(serde::de::Error::custom)
    }
}

mod ps256_private_key {

    use jwt_simple::prelude::PS256KeyPair;
    use serde::{self, Deserialize, Serializer, Deserializer};
    pub fn serialize<S>(
        key: &PS256KeyPair,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
    S: Serializer,
    {
        let s = key.to_pem().map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }
    
    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
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
