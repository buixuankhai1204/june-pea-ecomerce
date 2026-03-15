use crate::domain::model::{Order, OrderItem, OrderStatus};
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError};
use uuid::Uuid;

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create_order(
        &self,
        exec: &mut dyn DbExecutor,
        order: &Order,
        items: &[OrderItem],
    ) -> Result<(), AppError>;

    async fn get_order_by_id(&self, exec: &mut dyn DbExecutor, id: Uuid)
        -> Result<Order, AppError>;

    async fn update_order_status(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<(), AppError>;

    async fn list_orders(
        &self,
        exec: &mut dyn DbExecutor,
        customer_id: Uuid,
    ) -> Result<Vec<Order>, AppError>;

    async fn list_all_orders(&self, exec: &mut dyn DbExecutor) -> Result<Vec<Order>, AppError>;
}
