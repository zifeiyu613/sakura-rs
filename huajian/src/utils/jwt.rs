use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,    // 用户 ID
    pub exp: usize,     // 过期时间
    pub iat: usize,     // 签发时间
    pub role: String,   // 用户角色
}

pub fn generate_token(user_id: i64, role: &str, secret: &[u8], expiration: i64) -> Result<String, jsonwebtoken::errors::Error> {
    let now = OffsetDateTime::now_utc();
    let expires_at = now + Duration::seconds(expiration);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expires_at.unix_timestamp() as usize,
        iat: now.unix_timestamp() as usize,
        role: role.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
