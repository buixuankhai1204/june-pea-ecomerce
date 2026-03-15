use crate::domain::model::User;
use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use std::sync::Arc;

pub struct ListUsersUsecase {
    repo: Arc<dyn UserRepository>,
}

impl ListUsersUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<User>, AppError> {
        self.repo.list_users().await
    }
}
