use crate::AppError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_STAFF: &str = "staff";
pub const ROLE_CUSTOMER: &str = "customer";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: Uuid,    // User ID
    pub exp: usize,   // Expiration time
    pub role: String, // "admin", "staff", "customer"
}

impl UserClaims {
    pub fn is_admin(&self) -> bool {
        self.role == ROLE_ADMIN
    }

    /// Admins count as staff.
    pub fn is_staff(&self) -> bool {
        self.is_admin() || self.role == ROLE_STAFF
    }

    /// Staff may touch anyone's data; everyone else only their own. `owner` is `None` for a
    /// record nobody owns (a guest's order), which only staff may touch.
    pub fn ensure_owner_or_staff(&self, owner: Option<Uuid>) -> Result<(), AppError> {
        if self.is_staff() || owner == Some(self.sub) {
            Ok(())
        } else {
            Err(AppError::Forbidden("You can only access your own data".into()))
        }
    }
}

/// The caller, whoever they are. Rejects with 401 when the request carries no valid login.
///
/// Reads the claims the gateway's auth middleware put in the request. If that middleware is
/// missing, every handler using these extractors answers 401, so a route that is wired up
/// without auth fails closed instead of open.
pub struct AuthUser(pub UserClaims);

/// A caller who is staff or an admin. 401 when not logged in, 403 for anyone else.
pub struct Staff(pub UserClaims);

/// A caller who is an admin. 401 when not logged in, 403 for anyone else.
pub struct Admin(pub UserClaims);

fn claims_of(parts: &Parts) -> Result<UserClaims, AppError> {
    parts
        .extensions
        .get::<UserClaims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        claims_of(parts).map(AuthUser)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Staff {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let claims = claims_of(parts)?;
        if claims.is_staff() {
            Ok(Staff(claims))
        } else {
            Err(AppError::Forbidden("Staff only".into()))
        }
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let claims = claims_of(parts)?;
        if claims.is_admin() {
            Ok(Admin(claims))
        } else {
            Err(AppError::Forbidden("Admins only".into()))
        }
    }
}

/// There is no fallback secret: tokens signed with a key everyone can read are worth nothing.
/// The gateway calls this at startup so a missing secret stops the server instead of
/// surfacing as a 500 on the first login.
pub fn check_jwt_secret() -> Result<(), &'static str> {
    jwt_secret().map(|_| ()).map_err(|_| "JWT_SECRET must be set to a non-empty value")
}

fn jwt_secret() -> Result<String, AppError> {
    match std::env::var("JWT_SECRET") {
        Ok(secret) if !secret.trim().is_empty() => Ok(secret),
        _ => {
            tracing::error!("JWT_SECRET is not set");
            Err(AppError::InternalServerError)
        }
    }
}

pub fn decode_token(token: &str) -> Result<UserClaims, AppError> {
    decode_token_with(&jwt_secret()?, token)
}

pub fn encode_token(claims: UserClaims) -> Result<String, AppError> {
    encode_token_with(&jwt_secret()?, claims)
}

fn decode_token_with(secret: &str, token: &str) -> Result<UserClaims, AppError> {
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

fn encode_token_with(secret: &str, claims: UserClaims) -> Result<String, AppError> {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_err| AppError::InternalServerError)
}
