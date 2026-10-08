//! Stock and suppliers kept in memory, for the unit and component tests. Behaves like the SQL:
//! a variant with no stock row reads as 0, and updating one is a silent no-op. Ignores the
//! executor, so pair it with `shared::testing::NoopUnitOfWork`.

use crate::domain::model::{Stock, Supplier};
use crate::domain::repository::InventoryRepository;
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError};
use std::collections::BTreeMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryInventoryRepository {
    stock: Mutex<BTreeMap<Uuid, i32>>,
    suppliers: Mutex<Vec<Supplier>>,
}

impl InMemoryInventoryRepository {
    /// Gives a variant a stock row, as if it had been seeded.
    pub fn set(&self, variant_id: Uuid, quantity: i32) {
        self.stock.lock().unwrap().insert(variant_id, quantity);
    }

    pub fn quantity(&self, variant_id: Uuid) -> Option<i32> {
        self.stock.lock().unwrap().get(&variant_id).copied()
    }
}

#[async_trait]
impl InventoryRepository for InMemoryInventoryRepository {
    async fn get_stock(&self, _exec: &mut dyn DbExecutor, id: Uuid) -> Result<i32, AppError> {
        Ok(self.quantity(id).unwrap_or(0))
    }

    async fn get_stock_for_update(&self, _exec: &mut dyn DbExecutor, id: Uuid) -> Result<i32, AppError> {
        Ok(self.quantity(id).unwrap_or(0))
    }

    async fn update_stock(&self, _exec: &mut dyn DbExecutor, id: Uuid, quantity: i32) -> Result<(), AppError> {
        if let Some(q) = self.stock.lock().unwrap().get_mut(&id) {
            *q = quantity;
        }
        Ok(())
    }

    async fn list_all_stocks(&self, _exec: &mut dyn DbExecutor) -> Result<Vec<Stock>, AppError> {
        Ok(self.stock.lock().unwrap().iter().map(|(&variant_id, &quantity)| Stock { variant_id, quantity }).collect())
    }

    async fn list_suppliers(&self, _exec: &mut dyn DbExecutor) -> Result<Vec<Supplier>, AppError> {
        Ok(self.suppliers.lock().unwrap().clone())
    }

    async fn create_supplier(&self, _exec: &mut dyn DbExecutor, s: Supplier) -> Result<(), AppError> {
        self.suppliers.lock().unwrap().push(s);
        Ok(())
    }

    async fn delete_supplier(&self, _exec: &mut dyn DbExecutor, id: Uuid) -> Result<(), AppError> {
        self.suppliers.lock().unwrap().retain(|s| s.id != id);
        Ok(())
    }

    async fn list_low_stock(&self, _exec: &mut dyn DbExecutor, threshold: i32) -> Result<Vec<Stock>, AppError> {
        Ok(self
            .stock
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, &q)| q < threshold)
            .map(|(&variant_id, &quantity)| Stock { variant_id, quantity })
            .collect())
    }
}
