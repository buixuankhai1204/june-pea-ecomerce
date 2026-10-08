//! `OrderLookup` over the ordering service's use cases. The only place payment names ordering.

use crate::domain::orders::{OrderLookup, OrderState, OrderSummary};
use async_trait::async_trait;
use ordering::domain::model::OrderStatus;
use ordering::usecase::{get_order::GetOrderUsecase, update_order_status::UpdateOrderStatusUsecase};
use shared::error::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct OrderingAdapter {
    get_order: Arc<GetOrderUsecase>,
    update_order_status: Arc<UpdateOrderStatusUsecase>,
}

impl OrderingAdapter {
    pub fn new(
        get_order: Arc<GetOrderUsecase>,
        update_order_status: Arc<UpdateOrderStatusUsecase>,
    ) -> Self {
        Self {
            get_order,
            update_order_status,
        }
    }
}

#[async_trait]
impl OrderLookup for OrderingAdapter {
    async fn find(&self, order_id: Uuid) -> Result<OrderSummary, AppError> {
        let order = self.get_order.execute(order_id).await?;
        Ok(OrderSummary {
            id: order.id,
            total: order.total,
            state: match order.status {
                OrderStatus::Pending => OrderState::Pending,
                OrderStatus::Cancelled => OrderState::Cancelled,
                OrderStatus::Completed => OrderState::Completed,
            },
        })
    }

    async fn mark_completed(&self, order_id: Uuid) -> Result<(), AppError> {
        self.update_order_status
            .execute(order_id, OrderStatus::Completed)
            .await
    }
}
