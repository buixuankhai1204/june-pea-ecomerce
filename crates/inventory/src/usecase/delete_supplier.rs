use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteSupplierUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl DeleteSupplierUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), AppError> {
        let repo = self.repo.clone();

        UnitOfWork::run_atomic(
            &*self.uow,
            Box::new(move |exec| {
                Box::pin(async move {
                    repo.delete_supplier(exec, id).await?;
                    Ok(())
                })
            }),
        )
        .await?;

        Ok(())
    }
}
