use crate::domain::repository::InventoryRepository;
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError, infrastructure::postgres::SqlxExecutor};
use sqlx::Row;
use uuid::Uuid;

pub struct PostgresInventoryRepository;

use crate::domain::model::{Stock, Supplier};

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

    async fn list_suppliers(&self, exec: &mut dyn DbExecutor) -> Result<Vec<Supplier>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let rows = sqlx::query("SELECT id, name, contact, location, status FROM inventory.suppliers")
            .fetch_all(&mut *executor.tx)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(rows
            .into_iter()
            .map(|r| Supplier {
                id: r.try_get("id").unwrap(),
                name: r.try_get("name").unwrap(),
                contact: r.try_get("contact").unwrap(),
                location: r.try_get("location").unwrap(),
                status: r.try_get("status").unwrap(),
            })
            .collect())
    }

    async fn create_supplier(&self, exec: &mut dyn DbExecutor, s: Supplier) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query("INSERT INTO inventory.suppliers (id, name, contact, location, status) VALUES ($1, $2, $3, $4, $5)")
            .bind(s.id)
            .bind(s.name)
            .bind(s.contact)
            .bind(s.location)
            .bind(s.status)
            .execute(&mut *executor.tx)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    async fn delete_supplier(&self, exec: &mut dyn DbExecutor, id: Uuid) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query("DELETE FROM inventory.suppliers WHERE id = $1")
            .bind(id)
            .execute(&mut *executor.tx)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    async fn list_low_stock(&self, exec: &mut dyn DbExecutor, threshold: i32) -> Result<Vec<Stock>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let rows = sqlx::query("SELECT variant_id, quantity FROM inventory.stock WHERE quantity < $1")
            .bind(threshold)
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
