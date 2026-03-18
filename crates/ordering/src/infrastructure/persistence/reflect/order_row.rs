use crate::domain::model::{Order, OrderStatus};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct OrderRow {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub status: String,
    pub total: i64,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<OrderRow> for Order {
    fn from(row: OrderRow) -> Self {
        Self {
            id: row.id,
            customer_id: row.customer_id,
            status: match row.status.as_str() {
                "cancelled" => OrderStatus::Cancelled,
                "completed" => OrderStatus::Completed,
                _ => OrderStatus::Pending,
            },
            total: row.total,
            note: row.note,
            created_at: row.created_at,
        }
    }
}
