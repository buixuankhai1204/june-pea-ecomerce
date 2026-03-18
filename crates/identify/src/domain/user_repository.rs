use crate::domain::model::{User, UserMembership};
use async_trait::async_trait;
use shared::error::AppError;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn update_user(&self, user: &User) -> Result<(), AppError>;
    async fn list_users(&self) -> Result<Vec<User>, AppError>;
    async fn list_staff(&self) -> Result<Vec<User>, AppError>;
    async fn list_memberships(&self) -> Result<Vec<UserMembership>, AppError>;
    async fn delete_user(&self, id: Uuid) -> Result<(), AppError>;
}
