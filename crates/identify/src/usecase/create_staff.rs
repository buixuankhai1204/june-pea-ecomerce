use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use std::sync::Arc;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

pub struct CreateStaffUsecase {
    repo: Arc<dyn UserRepository>,
}

impl CreateStaffUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, email: String, password: String) -> Result<crate::dto::staff::StaffMember, AppError> {
        if let Some(_) = self.repo.find_by_email(&email).await? {
             return Err(AppError::Conflict("User already exists".into()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::InternalServerError)?
            .to_string();
        
        let mut user = crate::domain::model::User::new(email, password_hash);
        user.role = "staff".to_string(); // Default to staff role

        self.repo.create_user(&user).await?;
        Ok(user.into())
    }
}
