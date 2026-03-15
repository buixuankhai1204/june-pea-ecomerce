use crate::domain::repository::InventoryRepository;
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError, infrastructure::postgres::SqlxExecutor};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresInventoryRepository;

use crate::domain::model::Stock;

#[async_trait]
impl InventoryRepository for PostgresInventoryRepository {
    async fn get_stock(&self, exec: &mut dyn DbExecutor, id: Uuid) -> Result<i32, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let row = sqlx::query("SELECT quantity FROM inventory.stock WHERE variant_id = $1")
            .bind(id)
            .fetch_optional(&mut *executor.tx)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(row.map(|r| r.try_get("quantity").unwrap_or(0)).unwrap_or(0))
    }

    async fn get_stock_for_update(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
    ) -> Result<i32, AppError> {
        // Safe downcast to our Shared SQLx Wrapper
        let executor = SqlxExecutor::from_executor(exec);

        let row =
            sqlx::query("SELECT quantity FROM inventory.stock WHERE variant_id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *executor.tx)
                .await
                .map_err(|e| AppError::Database(e))?;

        Ok(row.map(|r| r.try_get("quantity").unwrap_or(0)).unwrap_or(0))
    }

    async fn update_stock(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
        quantity: i32,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query("UPDATE inventory.stock SET quantity = $1 WHERE variant_id = $2")
            .bind(quantity)
            .bind(id)
            .execute(&mut *executor.tx)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    async fn list_all_stocks(&self, exec: &mut dyn DbExecutor) -> Result<Vec<Stock>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let rows = sqlx::query("SELECT variant_id, quantity FROM inventory.stock")
            .fetch_all(&mut *executor.tx)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(rows
            .into_iter()
            .map(|r| Stock {
                variant_id: r.try_get("variant_id").unwrap(),
                quantity: r.try_get("quantity").unwrap(),
            })
            .collect())
    }
}
