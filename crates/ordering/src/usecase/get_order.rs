use crate::domain::{model::Order, repository::OrderRepository};
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct GetOrderUsecase {
    repo: Arc<dyn OrderRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl GetOrderUsecase {
    pub fn new(repo: Arc<dyn OrderRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, order_id: Uuid) -> Result<Order, AppError> {
        let repo = self.repo.clone();

        let fetched_order = Arc::new(tokio::sync::Mutex::new(None));
        let fetched_order_clone = fetched_order.clone();

        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let order = repo.get_order_by_id(exec, order_id).await?;
                    *fetched_order_clone.lock().await = Some(order);
                    Ok(())
                })
            }))
            .await?;

        let order = fetched_order.lock().await.take().unwrap();
        Ok(order)
    }
}
