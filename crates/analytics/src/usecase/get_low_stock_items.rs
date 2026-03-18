use crate::domain::model::LowStockItem;
use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;
use sqlx::Row;

pub struct GetLowStockItemsUsecase {
    pool: Arc<PgPool>,
}

impl GetLowStockItemsUsecase {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn execute(&self) -> Result<Vec<LowStockItem>, AppError> {
        let rows = sqlx::query(
            "SELECT v.id as variant_id, p.name as product_name, v.name as variant_name, s.quantity
             FROM inventory.stock s
             JOIN catalog.product_variants v ON s.variant_id = v.id
             JOIN catalog.products p ON v.product_id = p.id
             WHERE s.quantity < 20
             ORDER BY s.quantity ASC"
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| LowStockItem {
            variant_id: r.try_get("variant_id").unwrap(),
            product_name: r.try_get("product_name").unwrap(),
            variant_name: r.try_get("variant_name").unwrap(),
            current_stock: r.try_get("quantity").unwrap(),
        }).collect())
    }
}
