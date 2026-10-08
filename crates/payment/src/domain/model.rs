use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Expired,
    Refunded,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Paid => "paid",
            Self::Failed => "failed",
            Self::Expired => "expired",
            Self::Refunded => "refunded",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "paid" => Self::Paid,
            "failed" => Self::Failed,
            "expired" => Self::Expired,
            "refunded" => Self::Refunded,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentProvider {
    VnPay,
}

impl PaymentProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VnPay => "vnpay",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "vnpay" => Self::VnPay,
            _ => Self::VnPay,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: Uuid,
    pub order_id: Uuid,
    pub txn_ref: String,
    pub provider: PaymentProvider,
    pub amount: i64,
    pub payment_url: String,
    pub status: PaymentStatus,
    pub customer_ip: Option<String>,
    pub response_code: Option<String>,
    pub transaction_status: Option<String>,
    pub transaction_no: Option<String>,
    pub bank_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

impl PaymentIntent {
    pub fn new(
        order_id: Uuid,
        amount: i64,
        payment_url: String,
        txn_ref: String,
        provider: PaymentProvider,
        expiry_minutes: i64,
        customer_ip: Option<String>,
    ) -> Self {
        Self::new_at(
            Utc::now(),
            order_id,
            amount,
            payment_url,
            txn_ref,
            provider,
            expiry_minutes,
            customer_ip,
        )
    }

    /// Same as `new`, with the creation time given. The payment URL carries this time as
    /// `vnp_CreateDate`, and a refund has to quote it back, so both must be the same instant.
    #[allow(clippy::too_many_arguments)]
    pub fn new_at(
        now: DateTime<Utc>,
        order_id: Uuid,
        amount: i64,
        payment_url: String,
        txn_ref: String,
        provider: PaymentProvider,
        expiry_minutes: i64,
        customer_ip: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            order_id,
            txn_ref,
            provider,
            amount,
            payment_url,
            status: PaymentStatus::Pending,
            customer_ip,
            response_code: None,
            transaction_status: None,
            transaction_no: None,
            bank_code: None,
            created_at: now,
            updated_at: now,
            expires_at: now + Duration::minutes(expiry_minutes),
            paid_at: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at && self.status == PaymentStatus::Pending
    }

    pub fn mark_paid(
        &mut self,
        response_code: Option<String>,
        transaction_status: Option<String>,
        transaction_no: Option<String>,
        bank_code: Option<String>,
    ) {
        self.status = PaymentStatus::Paid;
        self.response_code = response_code;
        self.transaction_status = transaction_status;
        self.transaction_no = transaction_no;
        self.bank_code = bank_code;
        self.updated_at = Utc::now();
        self.paid_at = Some(self.updated_at);
    }

    pub fn mark_failed(&mut self, response_code: Option<String>) {
        self.status = PaymentStatus::Failed;
        self.response_code = response_code;
        self.updated_at = Utc::now();
    }

    pub fn mark_refunded(&mut self) {
        self.status = PaymentStatus::Refunded;
        self.updated_at = Utc::now();
    }
}
