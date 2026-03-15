use crate::domain::user_repository::UserRepository;
// Giả sử bạn đã viết encode_token trong shared
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use shared::{
    auth::{encode_token, UserClaims},
    error::AppError,
};
use std::sync::Arc;

pub struct AuthUsecase {
    repo: Arc<dyn UserRepository>,
}

impl AuthUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn register(&self, email: String, password: String) -> Result<(), AppError> {
        // 1. Check if user exists
        if self.repo.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("Email already exists".into()));
        }

        // 2. Hash password
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::InternalServerError)?
            .to_string();

        // 3. Save user
        let user = crate::domain::model::User::new(email, password_hash);
        self.repo.create_user(&user).await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<String, AppError> {
        let user = self
            .repo
            .find_by_email(email)
            .await?
            .ok_or(AppError::Unauthorized("Invalid credentials".into()))?;

        // Verify password
        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AppError::InternalServerError)?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;

        // Generate Token — exp must be a UNIX timestamp
        let exp = chrono::Utc::now().timestamp() as usize + 24 * 3600;
        let claims = UserClaims {
            sub: user.id,
            role: user.role,
            exp,
        };
        encode_token(claims)
    }
}
