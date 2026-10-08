use crate::domain::{
    GatewayError, PaymentGateway, PaymentIntent, PaymentRepository, PaymentStatus, RefundRequest,
};
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, warn};
use uuid::Uuid;

pub struct RefundPaymentUsecase {
    repo: Arc<dyn PaymentRepository>,
    uow: Arc<dyn UnitOfWork>,
    gateway: Arc<dyn PaymentGateway>,
}

impl RefundPaymentUsecase {
    pub fn new(
        repo: Arc<dyn PaymentRepository>,
        uow: Arc<dyn UnitOfWork>,
        gateway: Arc<dyn PaymentGateway>,
    ) -> Self {
        Self { repo, uow, gateway }
    }

    /// Refunds a paid payment in full: asks the provider first, and only records the refund
    /// here once the provider has confirmed it. `requested_by` is who asked, for the provider's
    /// records.
    pub async fn execute(&self, order_id: Uuid, requested_by: &str) -> Result<(), AppError> {
        let intent = self.find_intent(order_id).await?.ok_or_else(|| {
            AppError::NotFound(format!("Payment for order {} not found", order_id))
        })?;

        if intent.status != PaymentStatus::Paid {
            return Err(AppError::Validation(
                "Only paid payments can be refunded".to_string(),
            ));
        }

        let receipt = self
            .gateway
            .refund(RefundRequest {
                txn_ref: intent.txn_ref.clone(),
                amount: intent.amount,
                transaction_no: intent.transaction_no.clone(),
                transaction_date: intent.created_at,
                requested_by: requested_by.to_string(),
                reason: format!("Hoan tien don hang {}", order_id),
            })
            .await
            .map_err(into_app_error)?;

        // The provider has refunded by now. If saving fails the customer has their money back
        // while we still say "paid", and retrying would ask for a second refund. That's the
        // dual-write problem again; until there's a reconciliation job the log has to be enough
        // to fix it by hand.
        self.mark_refunded(&intent).await.map_err(|e| {
            error!(
                %order_id,
                txn_ref = %intent.txn_ref,
                refund_transaction_no = %receipt.transaction_no,
                "provider refunded the payment but we could not record it: {e}"
            );
            e
        })
    }

    async fn find_intent(&self, order_id: Uuid) -> Result<Option<PaymentIntent>, AppError> {
        let repo = self.repo.clone();
        let holder = Arc::new(Mutex::new(None));
        let holder_clone = holder.clone();
        self.uow
            .run_read_only(Box::new(move |exec| {
                let repo = repo.clone();
                let holder = holder_clone.clone();
                Box::pin(async move {
                    *holder.lock().await = repo.find_by_order_id(exec, order_id).await?;
                    Ok(())
                })
            }))
            .await?;

        let found = holder.lock().await.take();
        Ok(found)
    }

    /// `update_status` overwrites the provider fields, so pass the paid ones back in.
    async fn mark_refunded(&self, intent: &PaymentIntent) -> Result<(), AppError> {
        let repo = self.repo.clone();
        let intent = intent.clone();
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    repo.update_status(
                        exec,
                        intent.id,
                        PaymentStatus::Refunded,
                        intent.response_code,
                        intent.transaction_status,
                        intent.transaction_no,
                        intent.bank_code,
                    )
                    .await
                })
            }))
            .await
    }
}

fn into_app_error(err: GatewayError) -> AppError {
    match err {
        GatewayError::Rejected { code, message } => {
            AppError::Conflict(format!("VNPay refused the refund ({code}): {message}"))
        }
        GatewayError::InProgress => {
            AppError::Conflict("VNPay is still processing the refund, check again later".into())
        }
        GatewayError::Unavailable(why) => {
            warn!("VNPay refund failed: {why}");
            AppError::BadGateway("Payment provider is unavailable".into())
        }
        GatewayError::InvalidResponse(why) => {
            error!("VNPay refund answer not trusted: {why}");
            AppError::BadGateway("Payment provider sent an invalid response".into())
        }
    }
}

// the gateway is mocked with mockall, payments live in the in-memory repository
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::gateway::MockPaymentGateway;
    use crate::domain::{PaymentProvider, RefundReceipt};
    use crate::infrastructure::persistence::memory::InMemoryPaymentRepository;
    use shared::testing::NoopUnitOfWork;

    fn paid_intent() -> PaymentIntent {
        let mut intent = pending_intent();
        intent.mark_paid(
            Some("00".into()),
            Some("00".into()),
            Some("14000001".into()),
            Some("NCB".into()),
        );
        intent
    }

    fn pending_intent() -> PaymentIntent {
        PaymentIntent::new(
            Uuid::new_v4(),
            150_000,
            "https://pay.example/checkout".into(),
            "123456ABCDEF".into(),
            PaymentProvider::VnPay,
            15,
            None,
        )
    }

    fn refunded_by_vnpay() -> Result<RefundReceipt, GatewayError> {
        Ok(RefundReceipt {
            transaction_no: "14000099".into(),
            message: "Refund Success".into(),
        })
    }

    /// a repo holding `intent`, and the use case on top of it
    fn setup(
        intent: Option<&PaymentIntent>,
        gateway: MockPaymentGateway,
    ) -> (RefundPaymentUsecase, Arc<InMemoryPaymentRepository>) {
        let repo = Arc::new(InMemoryPaymentRepository::default());
        if let Some(intent) = intent {
            repo.insert(intent.clone());
        }
        let usecase =
            RefundPaymentUsecase::new(repo.clone(), Arc::new(NoopUnitOfWork), Arc::new(gateway));
        (usecase, repo)
    }

    fn never_called() -> MockPaymentGateway {
        let mut gateway = MockPaymentGateway::new();
        gateway.expect_refund().never();
        gateway
    }

    #[tokio::test]
    async fn refunds_a_paid_payment_through_the_provider_then_records_it() {
        let intent = paid_intent();
        let mut gateway = MockPaymentGateway::new();
        let asked = intent.clone();
        gateway
            .expect_refund()
            .withf(move |r| {
                r.txn_ref == asked.txn_ref
                    && r.amount == 150_000
                    && r.transaction_no.as_deref() == Some("14000001")
                    && r.transaction_date == asked.created_at
                    && r.requested_by == "admin-1"
            })
            .times(1)
            .returning(|_| refunded_by_vnpay());
        let (usecase, repo) = setup(Some(&intent), gateway);

        usecase.execute(intent.order_id, "admin-1").await.unwrap();

        let after = repo.latest_for_order(intent.order_id).unwrap();
        assert_eq!(after.status, PaymentStatus::Refunded);
        // the provider fields of the original payment survive the status change
        assert_eq!(after.response_code.as_deref(), Some("00"));
        assert_eq!(after.transaction_no.as_deref(), Some("14000001"));
        assert_eq!(after.bank_code.as_deref(), Some("NCB"));
    }

    #[tokio::test]
    async fn a_payment_nobody_started_is_not_found_and_the_provider_is_not_asked() {
        let (usecase, _) = setup(None, never_called());

        let err = usecase.execute(Uuid::new_v4(), "admin-1").await.unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn only_paid_payments_can_be_refunded() {
        let mut already_refunded = paid_intent();
        already_refunded.mark_refunded();
        for not_paid in [pending_intent(), already_refunded] {
            let (usecase, repo) = setup(Some(&not_paid), never_called());

            let err = usecase
                .execute(not_paid.order_id, "admin-1")
                .await
                .unwrap_err();

            assert!(matches!(err, AppError::Validation(_)), "{:?}", not_paid.status);
            assert_eq!(repo.latest_for_order(not_paid.order_id).unwrap().status, not_paid.status);
        }
    }

    #[tokio::test]
    async fn a_refusal_from_the_provider_is_a_conflict_and_the_payment_stays_paid() {
        let intent = paid_intent();
        let mut gateway = MockPaymentGateway::new();
        gateway.expect_refund().returning(|_| {
            Err(GatewayError::Rejected {
                code: "94".into(),
                message: "Duplicate request".into(),
            })
        });
        let (usecase, repo) = setup(Some(&intent), gateway);

        let err = usecase.execute(intent.order_id, "admin-1").await.unwrap_err();

        assert!(matches!(&err, AppError::Conflict(m) if m.contains("94")), "{err:?}");
        assert_eq!(repo.latest_for_order(intent.order_id).unwrap().status, PaymentStatus::Paid);
    }

    #[tokio::test]
    async fn a_refund_still_in_progress_is_a_conflict_and_the_payment_stays_paid() {
        let intent = paid_intent();
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_refund()
            .returning(|_| Err(GatewayError::InProgress));
        let (usecase, repo) = setup(Some(&intent), gateway);

        let err = usecase.execute(intent.order_id, "admin-1").await.unwrap_err();

        assert!(matches!(err, AppError::Conflict(_)));
        assert_eq!(repo.latest_for_order(intent.order_id).unwrap().status, PaymentStatus::Paid);
    }

    #[tokio::test]
    async fn an_unreachable_or_untrustworthy_provider_is_a_bad_gateway_and_the_payment_stays_paid() {
        for failure in [
            GatewayError::Unavailable("connection refused".into()),
            GatewayError::InvalidResponse("bad signature".into()),
        ] {
            let intent = paid_intent();
            let mut gateway = MockPaymentGateway::new();
            gateway
                .expect_refund()
                .returning(move |_| Err(failure.clone()));
            let (usecase, repo) = setup(Some(&intent), gateway);

            let err = usecase.execute(intent.order_id, "admin-1").await.unwrap_err();

            assert!(matches!(err, AppError::BadGateway(_)));
            assert_eq!(repo.latest_for_order(intent.order_id).unwrap().status, PaymentStatus::Paid);
        }
    }

    #[tokio::test]
    async fn failing_to_record_a_refund_the_provider_made_is_reported() {
        let intent = paid_intent();
        let mut gateway = MockPaymentGateway::new();
        gateway
            .expect_refund()
            .times(1)
            .returning(|_| refunded_by_vnpay());
        let (usecase, repo) = setup(Some(&intent), gateway);
        repo.set_writes_fail(true);

        let err = usecase.execute(intent.order_id, "admin-1").await.unwrap_err();

        assert!(matches!(err, AppError::InternalServerError));
    }
}
