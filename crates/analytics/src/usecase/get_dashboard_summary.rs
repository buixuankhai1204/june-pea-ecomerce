use crate::domain::model::DashboardSummary;
use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;

pub struct GetDashboardSummaryUsecase {
    pool: Arc<PgPool>,
}

impl GetDashboardSummaryUsecase {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn execute(&self) -> Result<DashboardSummary, AppError> {
        // Mocked aggregation across various domains
        let total_revenue: i64 = sqlx::query_scalar("SELECT SUM(total) FROM ordering.orders WHERE status = 'completed'")
            .fetch_one(&*self.pool)
            .await.unwrap_or(0);

        let today_orders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ordering.orders WHERE created_at >= NOW() - INTERVAL '1 day'")
            .fetch_one(&*self.pool)
            .await.unwrap_or(0);

        let new_products: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM catalog.products WHERE created_at >= NOW() - INTERVAL '7 day'")
            .fetch_one(&*self.pool)
            .await.unwrap_or(0);

        Ok(DashboardSummary {
            total_revenue,
            today_orders: today_orders as usize,
            new_products_this_week: new_products as usize,
            stock_accuracy: 98.0, // Mocked for now
        })
    }
}
