//! The payment provider's server-to-server API, as the use cases see it. Customers pay through
//! the provider's own page and we hear about it by IPN callback, so the only thing we call is
//! the refund API. `infrastructure::vnpay::VnPayGateway` is the real one (HTTP), tests plug
//! in their own.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// A full refund of one payment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundRequest {
    pub txn_ref: String,
    /// in VND, the same unit as `PaymentIntent::amount`
    pub amount: i64,
    /// the provider's own id for the payment, from the IPN
    pub transaction_no: Option<String>,
    /// when the payment was created on our side (`PaymentIntent::created_at`)
    pub transaction_date: DateTime<Utc>,
    pub requested_by: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefundReceipt {
    /// the provider's id for the refund itself
    pub transaction_no: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GatewayError {
    /// The provider answered and said no (unknown transaction, duplicate request, ...).
    #[error("provider refused ({code}): {message}")]
    Rejected { code: String, message: String },
    /// The provider accepted the refund but hasn't finished it (it is with the bank).
    #[error("provider is still processing the refund")]
    InProgress,
    /// No usable answer: connection refused, timeout, HTTP error status.
    #[error("provider unavailable: {0}")]
    Unavailable(String),
    /// An answer we can't trust: not JSON, or the signature doesn't match.
    #[error("untrustworthy response from provider: {0}")]
    InvalidResponse(String),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PaymentGateway: Send + Sync {
    async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt, GatewayError>;
}
