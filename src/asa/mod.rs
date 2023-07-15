

use std::sync::{Arc, RwLock};
use std::time::{UNIX_EPOCH, SystemTime};

use jwt_simple::token::Token;
use jwt_simple::prelude::{Claims, PS256KeyPair, RSAKeyPairLike, PS256PublicKey, RSAPublicKeyLike, NoCustomClaims};
use serde::{Serialize, Deserialize};
use yandex_cloud::yandex::cloud::iam::v1::CreateIamTokenResponse;

use crate::auth::Credentials;



#[test]
fn bugoga() {

}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceAccountKey {
    pub id: String,
    pub service_account_id: String,
    pub created_at: String,
    pub key_algorithm: String,
    pub public_key: String,
    pub private_key: String,
}


#[test]
fn test_de() {
    use std::fs::File;
    use std::io::BufReader;
    let path = "test-env/authorized_key_4096.json";
    let file = File::open(path).unwrap();

    let reader = BufReader::new(file);
    let key: ServiceAccountKey = serde_json::from_reader(reader).unwrap();
    println!("public key: \n{}",key.public_key);
    println!("private key: \n{}", key.private_key);
    let token = make_jwt(&key);

    println!("token: {token}");

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
    let mut claims = Claims::create(Duration::from_mins(1))
        .with_issuer(&key.service_account_id)
        .with_audience("https://iam.api.cloud.yandex.net/iam/v1/tokens");

    let pair = PS256KeyPair::from_pem(&key.private_key).unwrap().with_key_id(&key.id);
    println!("padding scheme: {:?}", pair.padding_scheme());
    let token = pair.sign(claims).unwrap();
    let public = PS256PublicKey::from_pem(&key.public_key).unwrap();
    let claims = public.verify_token::<NoCustomClaims>(&token, None);
    println!("verified claims: {claims:?}");
    let meta = Token::decode_metadata(&token).unwrap();
    println!("meta: {meta:?}");
    token
}

async fn update_token() {
    
}

#[tokio::test]
async fn get_token() {
    use std::fs::File;
    use std::io::BufReader;
    let path = "test-env/authorized_key_4096.json";
    let file = File::open(path).unwrap();

    let reader = BufReader::new(file);
    let key: ServiceAccountKey = serde_json::from_reader(reader).unwrap();
    println!("public key: \n{}",key.public_key);
    let token = make_jwt(&key);
    println!("token: {token}");
    use crate::client;
    let url = "grpcs://iam.api.cloud.yandex.net:443";
    let endpoint = client::create_endpoint(url.try_into().unwrap());
    let channel = endpoint.connect_lazy();
    let mut client = yandex_cloud::yandex::cloud::iam::v1::iam_token_service_client::IamTokenServiceClient::new(channel);
    let request = yandex_cloud::yandex::cloud::iam::v1::CreateIamTokenRequest {
        identity: Some(yandex_cloud::yandex::cloud::iam::v1::create_iam_token_request::Identity::Jwt(token))
    };
    let resp = client.create(request).await.unwrap();
    let response = resp.into_inner();
    println!("response: {response:?}");
    let expires = response.expires_at.as_ref().unwrap();
    let expires = UNIX_EPOCH + std::time::Duration::from_secs(expires.seconds as u64);
    let after = expires.duration_since(SystemTime::now()).unwrap();
    println!("expires: {}",after.as_secs());


    use std::env::var;
    let url = var("YDB_URL").expect("YDB_URL not set");
    let db_name = var("DB_NAME").expect("DB_NAME not set");

    let endpoint = client::create_endpoint(url.try_into().unwrap());
    let channel = endpoint.connect_lazy();
    let mut service = crate::YdbConnection::new(channel, db_name.as_str().try_into().unwrap(), response.iam_token);
    
         // how to use it, e.g. use discovery service:
    use crate::generated::ydb::discovery::ListEndpointsRequest;
    let endpoints_response = service.discovery().list_endpoints(
        ListEndpointsRequest{
            database: db_name.into(), 
            ..Default::default()
        }
    ).await.unwrap();
    
    // how you can parse response to invoke result with YdbResponseWithResult trait
    use crate::YdbResponseWithResult;
    let endpoints_result = endpoints_response.get_ref().result().unwrap();
    println!("endpoints {:?}", endpoints_result.endpoints);
    assert!(endpoints_result.endpoints.len() > 0);
    
}


