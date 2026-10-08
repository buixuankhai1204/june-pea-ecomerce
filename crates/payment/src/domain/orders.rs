//! What payment needs from the ordering service, as payment sees it. Payment doesn't use
//! ordering's types: `infrastructure::ordering::OrderingAdapter` translates, and tests fake
//! this trait instead of the ordering internals.

use async_trait::async_trait;
use shared::error::AppError;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderState {
    Pending,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderSummary {
    pub id: Uuid,
    /// in VND
    pub total: i64,
    pub state: OrderState,
}

#[async_trait]
pub trait OrderLookup: Send + Sync {
    /// `AppError::NotFound` when there is no such order.
    async fn find(&self, order_id: Uuid) -> Result<OrderSummary, AppError>;
    /// The customer has paid: the order is done as far as payment is concerned.
    async fn mark_completed(&self, order_id: Uuid) -> Result<(), AppError>;
}
