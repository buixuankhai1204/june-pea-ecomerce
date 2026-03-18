use crate::domain::model::Supplier;
use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateSupplierUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl CreateSupplierUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, name: String, contact: String, location: String) -> Result<(), AppError> {
        let repo = self.repo.clone();
        let s = Supplier {
            id: Uuid::new_v4(),
            name,
            contact,
            location,
            status: "Active".to_string(),
        };

        UnitOfWork::run_atomic(
            &*self.uow,
            Box::new(move |exec| {
                Box::pin(async move {
                    repo.create_supplier(exec, s).await?;
                    Ok(())
                })
            }),
        )
        .await?;

        Ok(())
    }
}
