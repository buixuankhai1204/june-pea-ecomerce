use crate::domain::model::User;
use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetMeUsecase {
    repo: Arc<dyn UserRepository>,
}

impl GetMeUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<User, AppError> {
        self.repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))
    }
}
