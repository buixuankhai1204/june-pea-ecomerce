use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use uuid::Uuid;
use std::sync::Arc;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

pub struct ChangePasswordUsecase {
    repo: Arc<dyn UserRepository>,
}

impl ChangePasswordUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, user_id: Uuid, new_password: String) -> Result<(), AppError> {
        let mut user = self.repo.find_by_id(user_id).await?
            .ok_or(AppError::NotFound("User not found".into()))?;

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|_| AppError::InternalServerError)?
            .to_string();

        user.password_hash = password_hash;
        self.repo.update_user(&user).await
    }
}
