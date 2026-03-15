use crate::domain::{PaymentRepository, PaymentStatus};
use crate::infrastructure::vnpay::VnPayClient;
use ordering::domain::model::OrderStatus as OrderDomainStatus;
use ordering::usecase::update_order_status::UpdateOrderStatusUsecase;
use serde::Serialize;
use shared::{database::UnitOfWork, error::AppError};
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct IpnResponseBody {
    pub rsp_code: String,
    pub message: String,
}

pub struct HandleVnPayIpnUsecase {
    repo: Arc<dyn PaymentRepository>,
    uow: Arc<dyn UnitOfWork>,
    vn_pay_client: Arc<VnPayClient>,
    update_order_status: Arc<UpdateOrderStatusUsecase>,
}

impl HandleVnPayIpnUsecase {
    pub fn new(
        repo: Arc<dyn PaymentRepository>,
        uow: Arc<dyn UnitOfWork>,
        vn_pay_client: Arc<VnPayClient>,
        update_order_status: Arc<UpdateOrderStatusUsecase>,
    ) -> Self {
        Self {
            repo,
            uow,
            vn_pay_client,
            update_order_status,
        }
    }

    pub async fn execute(
        &self,
        raw_params: BTreeMap<String, String>,
    ) -> Result<IpnResponseBody, AppError> {
        if !self.vn_pay_client.verify_signature(&raw_params) {
            return Ok(IpnResponseBody::error("97", "Invalid signature"));
        }

        let txn_ref = raw_params
            .get("vnp_TxnRef")
            .cloned()
            .ok_or_else(|| AppError::Validation("Missing vnp_TxnRef".into()))?;

        let amount_param = raw_params
            .get("vnp_Amount")
            .and_then(|a| a.parse::<i64>().ok())
            .unwrap_or_default();
        let amount = amount_param / 100;

        let response_code = raw_params
            .get("vnp_ResponseCode")
            .cloned()
            .unwrap_or_default();
        let transaction_status = raw_params
            .get("vnp_TransactionStatus")
            .cloned()
            .unwrap_or_default();

        let payment = self.fetch_by_txn_ref(txn_ref.clone()).await?;

        if payment.amount != amount {
            return Ok(IpnResponseBody::error("04", "Amount mismatch"));
        }

        if response_code == "00" && transaction_status == "00" {
            if payment.status != PaymentStatus::Paid {
                self.mark_status(
                    payment.id,
                    PaymentStatus::Paid,
                    Some(response_code.clone()),
                    Some(transaction_status.clone()),
                    raw_params.get("vnp_TransactionNo").cloned(),
                    raw_params.get("vnp_BankCode").cloned(),
                )
                .await?;

                self.update_order_status
                    .execute(payment.order_id, OrderDomainStatus::Completed)
                    .await?;
            }
            return Ok(IpnResponseBody::success());
        }

        if payment.status == PaymentStatus::Pending {
            self.mark_status(
                payment.id,
                PaymentStatus::Failed,
                Some(response_code.clone()),
                Some(transaction_status.clone()),
                raw_params.get("vnp_TransactionNo").cloned(),
                raw_params.get("vnp_BankCode").cloned(),
            )
            .await?;
        }

        Ok(IpnResponseBody::error(&response_code, "Payment failed"))
    }

    async fn fetch_by_txn_ref(
        &self,
        txn_ref: String,
    ) -> Result<crate::domain::PaymentIntent, AppError> {
        let repo = self.repo.clone();
        let holder = Arc::new(tokio::sync::Mutex::new(None));
        let holder_clone = holder.clone();
        self.uow
            .run_read_only(Box::new(move |exec| {
                let repo = repo.clone();
                let holder = holder_clone.clone();
                let txn_ref = txn_ref.clone();
                Box::pin(async move {
                    let payment = repo.find_by_txn_ref(exec, &txn_ref).await?;
                    *holder.lock().await = payment;
                    Ok(())
                })
            }))
            .await?;

        let result = holder.lock().await.take();
        result.ok_or_else(|| AppError::NotFound("Payment not found".into()))
    }

    async fn mark_status(
        &self,
        payment_id: Uuid,
        status: PaymentStatus,
        response_code: Option<String>,
        transaction_status: Option<String>,
        transaction_no: Option<String>,
        bank_code: Option<String>,
    ) -> Result<(), AppError> {
        let repo = self.repo.clone();
        self.uow
            .run_atomic(Box::new(move |exec| {
                let repo = repo.clone();
                Box::pin(async move {
                    repo.update_status(
                        exec,
                        payment_id,
                        status,
                        response_code,
                        transaction_status,
                        transaction_no,
                        bank_code,
                    )
                    .await
                })
            }))
            .await
    }
}

impl IpnResponseBody {
    pub fn success() -> Self {
        Self {
            rsp_code: "00".into(),
            message: "Success".into(),
        }
    }

    pub fn error(code: &str, message: &str) -> Self {
        Self {
            rsp_code: code.into(),
            message: message.into(),
        }
    }
}
