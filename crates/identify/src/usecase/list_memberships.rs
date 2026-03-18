use crate::domain::model::UserMembership;
use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use std::sync::Arc;

pub struct ListMembershipsUsecase {
    repo: Arc<dyn UserRepository>,
}

impl ListMembershipsUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<UserMembership>, AppError> {
        self.repo.list_memberships().await
    }
}
