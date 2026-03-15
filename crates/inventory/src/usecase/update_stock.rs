use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateStockUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl UpdateStockUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, variant_id: Uuid, quantity: i32) -> Result<(), AppError> {
        let repo = self.repo.clone();
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move { repo.update_stock(exec, variant_id, quantity).await })
            }))
            .await
    }
}
