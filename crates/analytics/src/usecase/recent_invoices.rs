use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct Invoice {
    pub order_id: Uuid,
    pub customer_email: String,
    pub total: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

pub struct GetRecentInvoicesUsecase {
    pool: Arc<PgPool>,
}

impl GetRecentInvoicesUsecase {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn execute(&self) -> Result<Vec<Invoice>, AppError> {
        let rows = sqlx::query(
            "SELECT o.id as order_id, u.email as customer_email, o.total, o.status, o.created_at
             FROM ordering.orders o
             LEFT JOIN identify.users u ON o.customer_id = u.id
             ORDER BY o.created_at DESC
             LIMIT 10"
        )
        .fetch_all(&*self.pool)
        .await?;

        use sqlx::Row;
        Ok(rows.into_iter().map(|r| Invoice {
            order_id: r.try_get("order_id").unwrap(),
            customer_email: r.try_get("customer_email").unwrap_or_else(|_| "Guest".to_string()),
            total: r.try_get("total").unwrap(),
            status: r.try_get("status").unwrap(),
            created_at: r.try_get("created_at").unwrap(),
        }).collect())
    }
}
