//! Payments kept in memory. What the unit and component tests use instead of Postgres, the
//! way `PostgresPaymentRepository` is what the integration tests use. It ignores the executor
//! it is given, so pair it with `shared::testing::NoopUnitOfWork`.

use crate::domain::{PaymentIntent, PaymentRepository, PaymentStatus};
use async_trait::async_trait;
use chrono::Utc;
use shared::{database::DbExecutor, error::AppError};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryPaymentRepository {
    intents: Mutex<HashMap<Uuid, PaymentIntent>>,
    writes_fail: AtomicBool,
}

impl InMemoryPaymentRepository {
    /// Puts a payment in, as if an earlier request had created it.
    pub fn insert(&self, intent: PaymentIntent) {
        self.intents.lock().unwrap().insert(intent.id, intent);
    }

    /// The newest payment for an order, for a test to look at afterwards.
    pub fn latest_for_order(&self, order_id: Uuid) -> Option<PaymentIntent> {
        self.intents
            .lock()
            .unwrap()
            .values()
            .filter(|p| p.order_id == order_id)
            .max_by_key(|p| p.created_at)
            .cloned()
    }

    /// Makes every write fail like a database that went away. Reads keep working.
    pub fn set_writes_fail(&self, fail: bool) {
        self.writes_fail.store(fail, Ordering::SeqCst);
    }

    fn check_writable(&self) -> Result<(), AppError> {
        if self.writes_fail.load(Ordering::SeqCst) {
            return Err(AppError::InternalServerError);
        }
        Ok(())
    }
}

#[async_trait]
impl PaymentRepository for InMemoryPaymentRepository {
    async fn create_intent(
        &self,
        _exec: &mut dyn DbExecutor,
        intent: &PaymentIntent,
    ) -> Result<(), AppError> {
        self.check_writable()?;
        self.insert(intent.clone());
        Ok(())
    }

    async fn update_intent(
        &self,
        _exec: &mut dyn DbExecutor,
        intent: &PaymentIntent,
    ) -> Result<(), AppError> {
        self.check_writable()?;
        self.insert(intent.clone());
        Ok(())
    }

    async fn find_by_order_id(
        &self,
        _exec: &mut dyn DbExecutor,
        order_id: Uuid,
    ) -> Result<Option<PaymentIntent>, AppError> {
        Ok(self.latest_for_order(order_id))
    }

    async fn find_by_txn_ref(
        &self,
        _exec: &mut dyn DbExecutor,
        txn_ref: &str,
    ) -> Result<Option<PaymentIntent>, AppError> {
        Ok(self
            .intents
            .lock()
            .unwrap()
            .values()
            .find(|p| p.txn_ref == txn_ref)
            .cloned())
    }

    /// Same as the SQL: the provider fields are overwritten with what's passed in, `paid_at`
    /// is stamped when the new status is paid and kept otherwise.
    async fn update_status(
        &self,
        _exec: &mut dyn DbExecutor,
        id: Uuid,
        status: PaymentStatus,
        response_code: Option<String>,
        transaction_status: Option<String>,
        transaction_no: Option<String>,
        bank_code: Option<String>,
    ) -> Result<(), AppError> {
        self.check_writable()?;
        if let Some(p) = self.intents.lock().unwrap().get_mut(&id) {
            if status == PaymentStatus::Paid {
                p.paid_at = Some(Utc::now());
            }
            p.status = status;
            p.response_code = response_code;
            p.transaction_status = transaction_status;
            p.transaction_no = transaction_no;
            p.bank_code = bank_code;
            p.updated_at = Utc::now();
        }
        Ok(())
    }
}
