use crate::AppError;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: Uuid,    // User ID
    pub exp: usize,   // Expiration time
    pub role: String, // "admin", "customer"
}

pub fn decode_token(token: &str) -> Result<UserClaims, AppError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());

    let validation = Validation::new(Algorithm::HS256);

    decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|err| match err.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::Unauthorized("Token has expired".into())
        }
        jsonwebtoken::errors::ErrorKind::InvalidToken => {
            AppError::Unauthorized("Invalid token".into())
        }
        _ => AppError::Unauthorized("Could not validate credentials".into()),
    })
}

pub fn encode_token(claims: UserClaims) -> Result<String, AppError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_err| AppError::InternalServerError)
}
