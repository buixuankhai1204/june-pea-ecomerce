use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct DecreaseStockUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl DecreaseStockUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, variant_id: Uuid, amount: i32) -> Result<(), AppError> {
        let repo = self.repo.clone();

        // Execute inside a Shared Transaction
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let current = repo.get_stock_for_update(exec, variant_id).await?;

                    if current < amount {
                        return Err(AppError::Conflict("Insufficient stock".into()));
                    }

                    repo.update_stock(exec, variant_id, current - amount)
                        .await?;
                    Ok(())
                })
            }))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::memory::InMemoryInventoryRepository;
    use shared::testing::NoopUnitOfWork;

    fn usecase_with(variant: Uuid, quantity: i32) -> (DecreaseStockUsecase, Arc<InMemoryInventoryRepository>) {
        let repo = Arc::new(InMemoryInventoryRepository::default());
        repo.set(variant, quantity);
        (DecreaseStockUsecase::new(repo.clone(), Arc::new(NoopUnitOfWork)), repo)
    }

    #[tokio::test]
    async fn takes_the_amount_off_the_stock() {
        let variant = Uuid::new_v4();
        let (usecase, repo) = usecase_with(variant, 10);

        usecase.execute(variant, 4).await.unwrap();

        assert_eq!(repo.quantity(variant), Some(6));
    }

    #[tokio::test]
    async fn can_take_the_last_unit() {
        let variant = Uuid::new_v4();
        let (usecase, repo) = usecase_with(variant, 3);

        usecase.execute(variant, 3).await.unwrap();

        assert_eq!(repo.quantity(variant), Some(0));
    }

    #[tokio::test]
    async fn refuses_to_go_below_zero_and_leaves_the_stock_alone() {
        let variant = Uuid::new_v4();
        let (usecase, repo) = usecase_with(variant, 3);

        let err = usecase.execute(variant, 4).await.unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
        assert_eq!(repo.quantity(variant), Some(3));
    }

    #[tokio::test]
    async fn a_variant_with_no_stock_row_counts_as_out_of_stock() {
        let (usecase, _) = usecase_with(Uuid::new_v4(), 5);

        let err = usecase.execute(Uuid::new_v4(), 1).await.unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
    }
}
