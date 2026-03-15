use crate::domain::{model::Order, repository::OrderRepository};
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct ListOrdersUsecase {
    repo: Arc<dyn OrderRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl ListOrdersUsecase {
    pub fn new(repo: Arc<dyn OrderRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, customer_id: Uuid) -> Result<Vec<Order>, AppError> {
        let repo = self.repo.clone();

        let fetched_orders = Arc::new(tokio::sync::Mutex::new(None));
        let fetched_orders_clone = fetched_orders.clone();

        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let orders = repo.list_orders(exec, customer_id).await?;
                    *fetched_orders_clone.lock().await = Some(orders);
                    Ok(())
                })
            }))
            .await?;

        let orders = fetched_orders.lock().await.take().unwrap();
        Ok(orders)
    }
}
