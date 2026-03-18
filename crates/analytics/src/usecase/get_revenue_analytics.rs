use crate::domain::model::RevenuePoint;
use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;

pub struct GetRevenueAnalyticsUsecase {
    pool: Arc<PgPool>,
}

impl GetRevenueAnalyticsUsecase {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn execute(&self) -> Result<Vec<RevenuePoint>, AppError> {
        // Return dummy data for a week by day for now
        let mut points = Vec::new();
        points.push(RevenuePoint { label: "Mon".to_string(), value: 1250000 });
        points.push(RevenuePoint { label: "Tue".to_string(), value: 890000 });
        points.push(RevenuePoint { label: "Wed".to_string(), value: 1650000 });
        points.push(RevenuePoint { label: "Thu".to_string(), value: 1420000 });
        points.push(RevenuePoint { label: "Fri".to_string(), value: 2100000 });
        points.push(RevenuePoint { label: "Sat".to_string(), value: 2450000 });
        points.push(RevenuePoint { label: "Sun".to_string(), value: 1980000 });
        Ok(points)
    }
}
