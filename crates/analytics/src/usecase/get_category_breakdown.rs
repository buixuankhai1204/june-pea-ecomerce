use crate::domain::model::CategoryBreakdown;
use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;

pub struct GetCategoryBreakdownUsecase {
    pool: Arc<PgPool>,
}

impl GetCategoryBreakdownUsecase {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn execute(&self) -> Result<Vec<CategoryBreakdown>, AppError> {
        let mut breakdown = Vec::new();
        breakdown.push(CategoryBreakdown { category_name: "Clothing".to_string(), sales_percentage: 42.0, color: "#6366F1".to_string() });
        breakdown.push(CategoryBreakdown { category_name: "Electronics".to_string(), sales_percentage: 31.0, color: "#F59E0B".to_string() });
        breakdown.push(CategoryBreakdown { category_name: "Food".to_string(), sales_percentage: 15.0, color: "#10B981".to_string() });
        breakdown.push(CategoryBreakdown { category_name: "Others".to_string(), sales_percentage: 12.0, color: "#F43F5E".to_string() });
        Ok(breakdown)
    }
}
