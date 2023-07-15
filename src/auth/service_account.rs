

use std::sync::{Arc, RwLock};
use std::time::{UNIX_EPOCH, SystemTime};


use jwt_simple::prelude::{Claims, PS256KeyPair, PS256PublicKey, RSAKeyPairLike};
use serde::{Serialize, Deserialize};
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
        let mut response = request_iam_token(&key).await?;
        let token = Arc::new(RwLock::new(response.iam_token.clone()));
        let result = Self {token};
        let token = Arc::downgrade(&result.token);
        tokio::spawn(async move {
            loop {
                let sleep_duration = invoke_sleep_duration(&response);
                tokio::time::sleep(sleep_duration).await;
                response = request_iam_token(&key).await.unwrap();
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

fn invoke_sleep_duration(response: &CreateIamTokenResponse) -> tokio::time::Duration {
    use std::time::Duration;
    let CreateIamTokenResponse {iam_token: _, expires_at} = response;
    let expires = if let Some(ts) = expires_at {
        (UNIX_EPOCH + Duration::from_secs(ts.seconds as u64)) //convert ts to SystemTime
        .duration_since(SystemTime::now() + Duration::from_secs(60)) //расчитываем время сна (с запасом)
        .unwrap_or_default()
    } else  {
        Duration::from_secs(50)
    };
    expires.into()
}

pub async fn request_iam_token(key: &ServiceAccountKey) -> Result<CreateIamTokenResponse, tonic::Status> {
    let jwt = make_jwt(key);
    let url = "grpcs://iam.api.cloud.yandex.net:443";
    let endpoint = crate::client::create_endpoint(url.try_into().unwrap());
    let mut client = yandex_cloud::yandex::cloud::iam::v1::iam_token_service_client::IamTokenServiceClient::new(endpoint.connect_lazy());
    let request = yandex_cloud::yandex::cloud::iam::v1::CreateIamTokenRequest {
        identity: Some(yandex_cloud::yandex::cloud::iam::v1::create_iam_token_request::Identity::Jwt(jwt))
    };
    let resp = client.create(request).await?;
    Ok(resp.into_inner())
}

fn make_jwt(key: &ServiceAccountKey) -> String {
    use jwt_simple::prelude::Duration;
    let claims = Claims::create(Duration::from_mins(1))
    .with_issuer(&key.service_account_id)
    .with_audience("https://iam.api.cloud.yandex.net/iam/v1/tokens");
    let pair = key.private_key.clone().with_key_id(&key.id);
    let token = pair.sign(claims).unwrap();
    token
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
