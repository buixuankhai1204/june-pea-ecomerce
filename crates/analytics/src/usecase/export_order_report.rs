use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;

pub struct ExportOrderReportUsecase {
    pool: Arc<PgPool>,
}

impl ExportOrderReportUsecase {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn execute(&self) -> Result<String, AppError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, customer_id, status, total, created_at
            FROM ordering.orders
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let mut csv = String::from("order_id,customer_id,status,total,created_at\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                row.id,
                row.customer_id.map(|id| id.to_string()).unwrap_or_default(),
                row.status,
                row.total,
                row.created_at
            ));
        }

        Ok(csv)
    }
}
