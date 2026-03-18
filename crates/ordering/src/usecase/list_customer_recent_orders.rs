use crate::domain::model::Order;
use crate::domain::repository::OrderRepository;
use shared::{database::UnitOfWork, error::AppError};
use uuid::Uuid;
use std::sync::Arc;

pub struct ListCustomerRecentOrdersUsecase {
    repo: Arc<dyn OrderRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl ListCustomerRecentOrdersUsecase {
    pub fn new(repo: Arc<dyn OrderRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, customer_id: Uuid, limit: i64) -> Result<Vec<Order>, AppError> {
        let repo = self.repo.clone();
        let fetched_orders = Arc::new(tokio::sync::Mutex::new(None));
        let fetched_orders_clone = fetched_orders.clone();

        self.uow.run_atomic(Box::new(move |exec| {
            Box::pin(async move {
                let orders = repo.list_customer_recent_orders(exec, customer_id, limit).await?;
                *fetched_orders_clone.lock().await = Some(orders);
                Ok(())
            })
        })).await?;

        let orders = fetched_orders.lock().await.take().unwrap();
        Ok(orders)
    }
}
