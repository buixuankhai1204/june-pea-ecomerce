use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct IncreaseStockUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl IncreaseStockUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, variant_id: Uuid, amount: i32) -> Result<(), AppError> {
        if amount <= 0 {
            return Err(AppError::Validation(
                "Amount must be greater than zero".into(),
            ));
        }

        let repo = self.repo.clone();

        // Execute inside a Shared Transaction
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let current = repo.get_stock_for_update(exec, variant_id).await?;

                    repo.update_stock(exec, variant_id, current + amount)
                        .await?;
                    Ok(())
                })
            }))
            .await
    }
}
