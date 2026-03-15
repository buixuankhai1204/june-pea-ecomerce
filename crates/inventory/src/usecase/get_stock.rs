use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct GetStockUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl GetStockUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, variant_id: Uuid) -> Result<i32, AppError> {
        let repo = self.repo.clone();

        let stock_value_arc = Arc::new(std::sync::atomic::AtomicI32::new(0));
        let stock_value_clone = stock_value_arc.clone();

        self.uow
            .run_atomic(Box::new(move |exec| {
                let stock_value_clone = stock_value_clone.clone();
                Box::pin(async move {
                    let current = repo.get_stock(exec, variant_id).await?;
                    stock_value_clone.store(current, std::sync::atomic::Ordering::SeqCst);
                    Ok(())
                })
            }))
            .await?;

        let final_value = stock_value_arc.load(std::sync::atomic::Ordering::SeqCst);
        Ok(final_value)
    }
}
