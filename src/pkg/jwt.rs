use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, encode, errors};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32,
    pub iat: u64,
    pub exp: u64,
    pub jti: String,
}

pub fn encode_token(
    sub: i32,
    expires_delta: chrono::Duration,
    secret: &str,
) -> errors::Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub,
        iat: now.timestamp() as u64,
        exp: (now + expires_delta).timestamp() as u64,
        jti: Uuid::new_v4().to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn decode_token(token: &str, secret: &str) -> errors::Result<Claims> {
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
}
