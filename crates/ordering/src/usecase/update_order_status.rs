use crate::domain::{model::OrderStatus, repository::OrderRepository};
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateOrderStatusUsecase {
    repo: Arc<dyn OrderRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl UpdateOrderStatusUsecase {
    pub fn new(repo: Arc<dyn OrderRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, order_id: Uuid, status: OrderStatus) -> Result<(), AppError> {
        let repo = self.repo.clone();
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move { repo.update_order_status(exec, order_id, status).await })
            }))
            .await
    }
}
