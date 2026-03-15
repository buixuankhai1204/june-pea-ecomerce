use crate::domain::model::Stock;
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError};
use uuid::Uuid;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn get_stock(&self, exec: &mut dyn DbExecutor, id: Uuid) -> Result<i32, AppError>;
    async fn get_stock_for_update(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
    ) -> Result<i32, AppError>;
    async fn update_stock(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
        quantity: i32,
    ) -> Result<(), AppError>;
    async fn list_all_stocks(&self, exec: &mut dyn DbExecutor) -> Result<Vec<Stock>, AppError>;
}
