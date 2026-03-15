use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateProfileUsecase {
    repo: Arc<dyn UserRepository>,
}

impl UpdateProfileUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, user_id: Uuid, email: String) -> Result<(), AppError> {
        let mut user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        user.email = email;
        self.repo.update_user(&user).await
    }
}
