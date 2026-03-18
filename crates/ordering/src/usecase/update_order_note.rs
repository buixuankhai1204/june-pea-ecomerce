use crate::domain::repository::OrderRepository;
use shared::{database::UnitOfWork, error::AppError};
use uuid::Uuid;
use std::sync::Arc;

pub struct UpdateOrderNoteUsecase {
    repo: Arc<dyn OrderRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl UpdateOrderNoteUsecase {
    pub fn new(repo: Arc<dyn OrderRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, order_id: Uuid, note: String) -> Result<(), AppError> {
        let repo = self.repo.clone();
        self.uow.run_atomic(Box::new(move |exec| {
            Box::pin(async move {
                repo.update_order_note(exec, order_id, note).await
            })
        })).await
    }
}
