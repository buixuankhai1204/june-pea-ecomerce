use crate::domain::model::PaymentStatus;
use crate::domain::repository::PaymentRepository;
use shared::error::AppError;
use uuid::Uuid;
use std::sync::Arc;

pub struct RefundPaymentUsecase {
    repo: Arc<dyn PaymentRepository>,
}

impl RefundPaymentUsecase {
    pub fn new(repo: Arc<dyn PaymentRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, order_id: Uuid) -> Result<(), AppError> {
        // In a real system, we'd call the payment provider API (e.g. VNPay) here.
        // For this task, we'll just update the status in our DB.
        
        struct DummyExecutor;
        impl shared::database::DbExecutor for DummyExecutor {}
        let mut exec = DummyExecutor;

        let intent = self.repo.find_by_order_id(&mut exec, order_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Payment for order {} not found", order_id)))?;

        if intent.status != crate::domain::model::PaymentStatus::Paid {
            return Err(AppError::Validation("Only paid payments can be refunded".to_string()));
        }

        self.repo.update_status(
            &mut exec,
            intent.id,
            PaymentStatus::Refunded,
            None, None, None, None
        ).await?;

        Ok(())
    }
}
