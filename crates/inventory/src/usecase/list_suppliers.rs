use crate::domain::model::Supplier;
use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;

pub struct ListSuppliersUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl ListSuppliersUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self) -> Result<Vec<Supplier>, AppError> {
        let suppliers = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let suppliers_clone = suppliers.clone();
        let repo = self.repo.clone();

        UnitOfWork::run_read_only(
            &*self.uow,
            Box::new(move |exec| {
                Box::pin(async move {
                    let mut guard = suppliers_clone.lock().await;
                    *guard = repo.list_suppliers(exec).await?;
                    Ok(())
                })
            }),
        )
        .await?;

        let guard = suppliers.lock().await;
        Ok(guard.clone())
    }
}
