use std::time::{SystemTime, UNIX_EPOCH};

use crate::{model::user, models::{Claims, TokenType}};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error};
use uuid::Uuid;

fn now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

pub fn generate_access_token(
    user_id: u64,
    username: String,
    role: u8,
    employee_id: Option<u64>,
    secret: &str,
    ttl: usize,
) -> String {
    let claims = Claims {
        user_id: user_id,
        sub: username,
        role: role,
        exp: now() + ttl,
        jti: Uuid::new_v4().to_string(),
        token_type: TokenType::Access,
        employee_id: employee_id,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

pub fn generate_refresh_token(
    user_id: u64,
    username: String,
    role: u8,
    employee_id: Option<u64>,
    secret: &str,
    ttl: usize,
) -> (String, Claims) {
    let claims = Claims {
        user_id: user_id,
        sub: username,
        role: role,
        exp: now() + ttl,
        jti: Uuid::new_v4().to_string(),
        token_type: TokenType::Refresh,
        employee_id: employee_id,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    (token, claims)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| e.to_string())
}
