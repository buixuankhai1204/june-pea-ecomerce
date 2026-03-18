use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use std::sync::Arc;

pub struct ListStaffUsecase {
    repo: Arc<dyn UserRepository>,
}

impl ListStaffUsecase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<crate::dto::staff::StaffMember>, AppError> {
        let users = self.repo.list_staff().await?;
        Ok(users.into_iter().map(Into::into).collect())
    }
}
