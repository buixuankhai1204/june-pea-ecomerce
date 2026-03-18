use crate::domain::rules::{
    order_item_quantity_must_be_positive::OrderItemQuantityMustBePositiveRule,
    order_must_be_pending_to_cancel::OrderMustBePendingToCancelRule,
    order_must_have_items::OrderMustHaveItemsRule,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{ensure, error::AppError};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Cancelled,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub status: OrderStatus,
    pub total: i64, // in cents
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub variant_id: Uuid,
    pub quantity: i32,
    pub unit_price: i64, // in cents
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewOrderItem {
    pub variant_id: Uuid,
    pub quantity: i32,
    pub unit_price: i64, // in cents
}

impl Order {
    /// Domain factory: validates all business rules then constructs Order + OrderItems.
    pub fn place(
        customer_id: Option<Uuid>,
        new_items: Vec<NewOrderItem>,
    ) -> Result<(Order, Vec<OrderItem>), AppError> {
        ensure(&OrderMustHaveItemsRule::new(new_items.len()))?;

        for item in &new_items {
            ensure(&OrderItemQuantityMustBePositiveRule::new(item.quantity))?;
        }

        let order_id = Uuid::new_v4();
        let total: i64 = new_items
            .iter()
            .map(|i| i.unit_price * i.quantity as i64)
            .sum();

        let order = Order {
            id: order_id,
            customer_id,
            status: OrderStatus::Pending,
            total,
            note: None,
            created_at: Utc::now(),
        };

        let items: Vec<OrderItem> = new_items
            .into_iter()
            .map(|i| OrderItem {
                id: Uuid::new_v4(),
                order_id,
                variant_id: i.variant_id,
                quantity: i.quantity,
                unit_price: i.unit_price,
            })
            .collect();

        Ok((order, items))
    }

    /// Domain mutator: validates cancellation rule then transitions status to Cancelled.
    pub fn cancel(&mut self) -> Result<(), AppError> {
        ensure(&OrderMustBePendingToCancelRule::new(self.status.clone()))?;
        self.status = OrderStatus::Cancelled;
        Ok(())
    }
}

// ─────────────────────────────────────────────
// Domain unit tests
// ─────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item(quantity: i32) -> NewOrderItem {
        NewOrderItem {
            variant_id: Uuid::new_v4(),
            quantity,
            unit_price: 1000, // $10.00
        }
    }

    // ── Order::place ──────────────────────────

    #[test]
    fn place_order_with_valid_items_succeeds() {
        let customer = Some(Uuid::new_v4());
        let items = vec![sample_item(2), sample_item(3)];
        let (order, order_items) = Order::place(customer, items).unwrap();

        assert_eq!(order.customer_id, customer);
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.total, 5000); // (2 + 3) * 1000
        assert_eq!(order_items.len(), 2);
        assert!(order_items.iter().all(|i| i.order_id == order.id));
    }

    #[test]
    fn place_order_with_no_items_fails() {
        let result = Order::place(Some(Uuid::new_v4()), vec![]);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn place_order_with_zero_quantity_fails() {
        let result = Order::place(Some(Uuid::new_v4()), vec![sample_item(0)]);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn place_order_with_negative_quantity_fails() {
        let result = Order::place(Some(Uuid::new_v4()), vec![sample_item(-1)]);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    // ── Order::cancel ─────────────────────────

    #[test]
    fn cancel_pending_order_succeeds() {
        let (mut order, _) = Order::place(Some(Uuid::new_v4()), vec![sample_item(1)]).unwrap();
        assert!(order.cancel().is_ok());
        assert_eq!(order.status, OrderStatus::Cancelled);
    }

    #[test]
    fn cancel_already_cancelled_order_fails() {
        let (mut order, _) = Order::place(Some(Uuid::new_v4()), vec![sample_item(1)]).unwrap();
        order.cancel().unwrap();
        let result = order.cancel();
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn cancel_completed_order_fails() {
        let (mut order, _) = Order::place(Some(Uuid::new_v4()), vec![sample_item(1)]).unwrap();
        order.status = OrderStatus::Completed;
        let result = order.cancel();
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
