

use std::{collections::HashMap, time::Instant};

use jwt_simple::{prelude::{Claims, PS256KeyPair, RSAKeyPairLike, Clock, PS256PublicKey, RSAPublicKeyLike, HS256Key, RS256KeyPair, NoCustomClaims}, token::Token};
use serde::{Serialize, Deserialize};
use yandex_cloud::yandex::cloud::iam::v1::{CreateIamTokenForServiceAccountRequest, CreateIamTokenRequest, create_iam_token_request::Identity};


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
    let path = "test-env/authorized_key.json";
    let file = File::open(path).unwrap();

    let reader = BufReader::new(file);
    let key: ServiceAccountKey = serde_json::from_reader(reader).unwrap();
    println!("public key: \n{}",key.public_key);
    println!("private key: \n{}", key.private_key);
    let token = make_token(&key);

    println!("token: {token}");

}

pub struct ServiceAccountCredential {
    key: String,
}

fn make_token(key: &ServiceAccountKey) -> String {
    use jwt_simple::prelude::Duration;
    //let mut headers = HashMap::new();
    //headers.insert("kid".to_owned(), key.id.clone());
    //let claims = Claims::with_custom_claims(headers, Duration::from_hours(1))
    let mut claims = Claims::create(Duration::from_hours(1))
        .with_issuer(&key.service_account_id)
        .with_audience("https://iam.api.cloud.yandex.net/iam/v1/tokens");

    //claims.issued_at = Some(now);
    //claims.expires_at = Some(exp);
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
    let path = "test-env/authorized_key.json";
    let file = File::open(path).unwrap();

    let reader = BufReader::new(file);
    let key: ServiceAccountKey = serde_json::from_reader(reader).unwrap();
    println!("public key: \n{}",key.public_key);
    let token = make_token(&key);
    println!("token: {token}");
    use crate::client;
    let url = "grpcs://iam.api.cloud.yandex.net:443";
    let endpoint = client::create_endpoint(url.try_into().unwrap());
    let channel = endpoint.connect_lazy();
    let mut client = yandex_cloud::yandex::cloud::iam::v1::iam_token_service_client::IamTokenServiceClient::new(channel);
    let request = CreateIamTokenRequest {
        identity: Some(Identity::Jwt(token))
    };
    let resp = client.create(request).await.unwrap();
    let response = resp.into_inner();
    println!("response: {response:?}");
    println!("expires: {} seconds", (response.expires_at.unwrap().seconds as u64 - 1689335892));
    
}


