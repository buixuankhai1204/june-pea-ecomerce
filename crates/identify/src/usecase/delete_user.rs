use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use uuid::Uuid;
use std::sync::Arc;

pub struct DeleteUserUsecase {
    repo: Arc<dyn UserRepository>,
}

impl DeleteUserUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete_user(id).await
    }
}
