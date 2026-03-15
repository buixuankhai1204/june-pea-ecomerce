use crate::domain::{model::Order, repository::OrderRepository};
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;

pub struct ListAllOrdersUsecase {
    repo: Arc<dyn OrderRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl ListAllOrdersUsecase {
    pub fn new(repo: Arc<dyn OrderRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self) -> Result<Vec<Order>, AppError> {
        let orders = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let orders_clone = orders.clone();
        let repo = self.repo.clone();

        self.uow
            .run_read_only(Box::new(move |exec| {
                Box::pin(async move {
                    let mut guard = orders_clone.lock().await;
                    *guard = repo.list_all_orders(exec).await?;
                    Ok(())
                })
            }))
            .await?;

        let guard = orders.lock().await;
        Ok(guard.clone())
    }
}
