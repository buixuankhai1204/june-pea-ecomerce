//! Orders kept in memory: what the unit and component tests use instead of Postgres. It ignores
//! the executor it is given, so pair it with `shared::testing::NoopUnitOfWork`.

use crate::domain::model::{Order, OrderItem, OrderStatus};
use crate::domain::repository::OrderRepository;
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryOrderRepository {
    orders: Mutex<HashMap<Uuid, Order>>,
    items: Mutex<HashMap<Uuid, Vec<OrderItem>>>,
}

impl InMemoryOrderRepository {
    pub fn items_of(&self, order_id: Uuid) -> Vec<OrderItem> {
        self.items.lock().unwrap().get(&order_id).cloned().unwrap_or_default()
    }

    /// newest first, like the SQL
    fn newest_first(&self, keep: impl Fn(&Order) -> bool) -> Vec<Order> {
        let mut orders: Vec<Order> = self
            .orders
            .lock()
            .unwrap()
            .values()
            .filter(|o| keep(o))
            .cloned()
            .collect();
        orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        orders
    }
}

fn not_found(id: Uuid) -> AppError {
    AppError::NotFound(format!("Order {id} not found"))
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn create_order(
        &self,
        _exec: &mut dyn DbExecutor,
        order: &Order,
        items: &[OrderItem],
    ) -> Result<(), AppError> {
        self.orders.lock().unwrap().insert(order.id, order.clone());
        self.items.lock().unwrap().insert(order.id, items.to_vec());
        Ok(())
    }

    async fn get_order_by_id(&self, _exec: &mut dyn DbExecutor, id: Uuid) -> Result<Order, AppError> {
        self.orders.lock().unwrap().get(&id).cloned().ok_or_else(|| not_found(id))
    }

    async fn update_order_status(
        &self,
        _exec: &mut dyn DbExecutor,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<(), AppError> {
        self.orders.lock().unwrap().get_mut(&id).ok_or_else(|| not_found(id))?.status = status;
        Ok(())
    }

    async fn list_orders(&self, _exec: &mut dyn DbExecutor, customer_id: Uuid) -> Result<Vec<Order>, AppError> {
        Ok(self.newest_first(|o| o.customer_id == Some(customer_id)))
    }

    async fn list_all_orders(&self, _exec: &mut dyn DbExecutor) -> Result<Vec<Order>, AppError> {
        Ok(self.newest_first(|_| true))
    }

    async fn update_order_note(&self, _exec: &mut dyn DbExecutor, id: Uuid, note: String) -> Result<(), AppError> {
        self.orders.lock().unwrap().get_mut(&id).ok_or_else(|| not_found(id))?.note = Some(note);
        Ok(())
    }

    async fn list_customer_recent_orders(
        &self,
        _exec: &mut dyn DbExecutor,
        customer_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Order>, AppError> {
        let mut orders = self.newest_first(|o| o.customer_id == Some(customer_id));
        orders.truncate(limit.max(0) as usize);
        Ok(orders)
    }
}
