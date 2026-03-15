use crate::domain::{PaymentIntent, PaymentStatus};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PaymentIntentView {
    pub payment_id: Uuid,
    pub order_id: Uuid,
    pub amount: i64,
    pub payment_url: String,
    pub qr_svg: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub status: PaymentStatus,
    pub txn_ref: String,
}

impl PaymentIntentView {
    pub fn from_intent(intent: PaymentIntent, qr_svg: Option<String>) -> Self {
        Self {
            payment_id: intent.id,
            order_id: intent.order_id,
            amount: intent.amount,
            payment_url: intent.payment_url,
            qr_svg,
            expires_at: intent.expires_at,
            status: intent.status,
            txn_ref: intent.txn_ref,
        }
    }
}
