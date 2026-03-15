use super::model::{PaymentIntent, PaymentStatus};
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError};
use uuid::Uuid;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn create_intent(
        &self,
        exec: &mut dyn DbExecutor,
        intent: &PaymentIntent,
    ) -> Result<(), AppError>;

    async fn update_intent(
        &self,
        exec: &mut dyn DbExecutor,
        intent: &PaymentIntent,
    ) -> Result<(), AppError>;

    async fn find_by_order_id(
        &self,
        exec: &mut dyn DbExecutor,
        order_id: Uuid,
    ) -> Result<Option<PaymentIntent>, AppError>;

    async fn find_by_txn_ref(
        &self,
        exec: &mut dyn DbExecutor,
        txn_ref: &str,
    ) -> Result<Option<PaymentIntent>, AppError>;

    async fn update_status(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
        status: PaymentStatus,
        response_code: Option<String>,
        transaction_status: Option<String>,
        transaction_no: Option<String>,
        bank_code: Option<String>,
    ) -> Result<(), AppError>;
}
