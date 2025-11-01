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

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::{decode_token, encode_token};

    #[test]
    fn encodes_and_decodes_token() {
        let token = encode_token(123, Duration::minutes(30), "secret").unwrap();
        let claims = decode_token(&token, "secret").unwrap();

        assert_eq!(claims.sub, 123);
        assert!(claims.exp > claims.iat);
        assert!(!claims.jti.is_empty());
    }

    #[test]
    fn rejects_wrong_secret() {
        let token = encode_token(123, Duration::minutes(30), "secret").unwrap();
        assert!(decode_token(&token, "wrong-secret").is_err());
    }

    #[test]
    fn rejects_expired_token() {
        let token = encode_token(123, Duration::hours(-1), "secret").unwrap();
        assert!(decode_token(&token, "secret").is_err());
    }
}
