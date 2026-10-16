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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn claims(role: &str) -> UserClaims {
        UserClaims { sub: Uuid::new_v4(), exp: usize::MAX / 2, role: role.into() }
    }

    fn parts_with(claims: Option<UserClaims>) -> Parts {
        let mut req = Request::new(());
        if let Some(c) = claims {
            req.extensions_mut().insert(c);
        }
        req.into_parts().0
    }

    #[test]
    fn admins_are_staff_but_customers_and_unknown_roles_are_not() {
        assert!(claims("admin").is_staff());
        assert!(claims("staff").is_staff());
        assert!(!claims("customer").is_staff());
        assert!(!claims("").is_staff());
        assert!(!claims("Admin").is_staff(), "roles are matched exactly");
    }

    #[test]
    fn only_admins_are_admins() {
        assert!(claims("admin").is_admin());
        assert!(!claims("staff").is_admin());
        assert!(!claims("customer").is_admin());
    }

    #[test]
    fn a_customer_may_only_touch_their_own_records() {
        let me = claims("customer");

        assert!(me.ensure_owner_or_staff(Some(me.sub)).is_ok());
        assert!(matches!(me.ensure_owner_or_staff(Some(Uuid::new_v4())), Err(AppError::Forbidden(_))));
        assert!(matches!(me.ensure_owner_or_staff(None), Err(AppError::Forbidden(_))));
    }

    #[test]
    fn staff_may_touch_anyones_records_including_guest_ones() {
        let staff = claims("staff");

        assert!(staff.ensure_owner_or_staff(Some(Uuid::new_v4())).is_ok());
        assert!(staff.ensure_owner_or_staff(None).is_ok());
    }

    #[tokio::test]
    async fn extractors_answer_401_without_a_login() {
        let mut parts = parts_with(None);

        assert!(matches!(AuthUser::from_request_parts(&mut parts, &()).await, Err(AppError::Unauthorized(_))));
        assert!(matches!(Staff::from_request_parts(&mut parts, &()).await, Err(AppError::Unauthorized(_))));
        assert!(matches!(Admin::from_request_parts(&mut parts, &()).await, Err(AppError::Unauthorized(_))));
    }

    #[tokio::test]
    async fn extractors_answer_403_to_a_caller_without_the_role() {
        let mut customer = parts_with(Some(claims("customer")));
        let mut staff = parts_with(Some(claims("staff")));

        assert!(AuthUser::from_request_parts(&mut customer, &()).await.is_ok());
        assert!(matches!(Staff::from_request_parts(&mut customer, &()).await, Err(AppError::Forbidden(_))));
        assert!(matches!(Admin::from_request_parts(&mut customer, &()).await, Err(AppError::Forbidden(_))));
        assert!(Staff::from_request_parts(&mut staff, &()).await.is_ok());
        assert!(matches!(Admin::from_request_parts(&mut staff, &()).await, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn admin_passes_every_extractor() {
        let mut parts = parts_with(Some(claims("admin")));

        assert!(AuthUser::from_request_parts(&mut parts, &()).await.is_ok());
        assert!(Staff::from_request_parts(&mut parts, &()).await.is_ok());
        assert!(Admin::from_request_parts(&mut parts, &()).await.is_ok());
    }

    #[test]
    fn a_token_round_trips_with_the_same_secret_and_not_with_another() {
        let original = claims("staff");

        let token = encode_token_with("secret-a", original.clone()).unwrap();

        let decoded = decode_token_with("secret-a", &token).unwrap();
        assert_eq!((decoded.sub, decoded.role.as_str()), (original.sub, "staff"));
        assert!(matches!(decode_token_with("secret-b", &token), Err(AppError::Unauthorized(_))));
    }

    #[test]
    fn an_expired_token_is_rejected() {
        let expired = UserClaims { sub: Uuid::new_v4(), exp: 1, role: "customer".into() };
        let token = encode_token_with("secret-a", expired).unwrap();

        assert!(matches!(decode_token_with("secret-a", &token), Err(AppError::Unauthorized(m)) if m.contains("expired")));
    }
}
