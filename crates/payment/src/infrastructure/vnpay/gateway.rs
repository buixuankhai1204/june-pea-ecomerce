//! `PaymentGateway` over HTTP: calls VNPay's merchant API with `vnp_Command=refund`.
//!
//! Requests and responses are signed with HMAC-SHA512 over the field values joined by `|`, in
//! the order VNPay documents (https://sandbox.vnpayment.vn/apis/docs/truy-van-hoan-tien/).
//! Tests run this against a wiremock server (`tests/integration_vnpay_gateway.rs`).

use super::{vnpay_timestamp, VnPayClient};
use crate::config::PaymentConfig;
use crate::domain::gateway::{GatewayError, PaymentGateway, RefundReceipt, RefundRequest};
use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha512;
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;

const API_VERSION: &str = "2.1.0";
/// `vnp_TransactionType`: 02 is a full refund, 03 a partial one
const FULL_REFUND: &str = "02";

/// The response fields VNPay signs, in the order it joins them.
const RESPONSE_SIGNED_FIELDS: [&str; 13] = [
    "vnp_ResponseId",
    "vnp_Command",
    "vnp_ResponseCode",
    "vnp_Message",
    "vnp_TmnCode",
    "vnp_TxnRef",
    "vnp_Amount",
    "vnp_BankCode",
    "vnp_PayDate",
    "vnp_TransactionNo",
    "vnp_TransactionType",
    "vnp_TransactionStatus",
    "vnp_OrderInfo",
];

pub struct VnPayGateway {
    http: reqwest::Client,
    api_url: String,
    tmn_code: String,
    hash_secret: String,
    server_ip: String,
}

impl VnPayGateway {
    pub fn new(config: &PaymentConfig) -> anyhow::Result<Self> {
        Self::with_timeout(config, Duration::from_secs(config.api_timeout_secs))
    }

    pub fn with_timeout(config: &PaymentConfig, timeout: Duration) -> anyhow::Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder().timeout(timeout).build()?,
            api_url: config.api_url.clone(),
            tmn_code: config.tmn_code.clone(),
            hash_secret: config.hash_secret.clone(),
            server_ip: config.server_ip.clone(),
        })
    }

    fn refund_body(&self, req: &RefundRequest) -> Value {
        let request_id = Uuid::new_v4().simple().to_string();
        let amount = (req.amount * 100).to_string();
        let transaction_no = req.transaction_no.clone().unwrap_or_default();
        let transaction_date = vnpay_timestamp(req.transaction_date);
        let create_date = vnpay_timestamp(Utc::now());

        let secure_hash = VnPayClient::sign(
            &self.hash_secret,
            &refund_signing_data(&[
                &request_id,
                API_VERSION,
                "refund",
                &self.tmn_code,
                FULL_REFUND,
                &req.txn_ref,
                &amount,
                &transaction_no,
                &transaction_date,
                &req.requested_by,
                &create_date,
                &self.server_ip,
                &req.reason,
            ]),
        );

        let mut body = json!({
            "vnp_RequestId": request_id,
            "vnp_Version": API_VERSION,
            "vnp_Command": "refund",
            "vnp_TmnCode": self.tmn_code,
            "vnp_TransactionType": FULL_REFUND,
            "vnp_TxnRef": req.txn_ref,
            "vnp_Amount": req.amount * 100,
            "vnp_OrderInfo": req.reason,
            "vnp_TransactionDate": transaction_date,
            "vnp_CreateBy": req.requested_by,
            "vnp_CreateDate": create_date,
            "vnp_IpAddr": self.server_ip,
            "vnp_SecureHash": secure_hash,
        });
        // optional, but part of the signed data either way (as an empty string)
        if let Some(no) = &req.transaction_no {
            body["vnp_TransactionNo"] = json!(no);
        }
        body
    }

    fn response_is_signed(&self, body: &Value) -> bool {
        let Some(Value::String(given)) = body.get("vnp_SecureHash") else {
            return false;
        };
        let Ok(given) = hex::decode(given) else {
            return false;
        };
        let mut mac = Hmac::<Sha512>::new_from_slice(self.hash_secret.as_bytes())
            .expect("HMAC accepts keys of any length");
        mac.update(response_signing_data(body).as_bytes());
        mac.verify_slice(&given).is_ok()
    }
}

#[async_trait]
impl PaymentGateway for VnPayGateway {
    async fn refund(&self, request: RefundRequest) -> Result<RefundReceipt, GatewayError> {
        let response = self
            .http
            .post(&self.api_url)
            .json(&self.refund_body(&request))
            .send()
            .await
            .map_err(|e| {
                let e = e.without_url();
                if e.is_timeout() {
                    GatewayError::Unavailable("timed out waiting for VNPay".into())
                } else {
                    GatewayError::Unavailable(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(GatewayError::Unavailable(format!(
                "VNPay answered HTTP {status}"
            )));
        }
        let body: Value = response
            .json()
            .await
            .map_err(|e| GatewayError::InvalidResponse(format!("body is not JSON: {e}")))?;

        let code = field(&body, "vnp_ResponseCode");
        if code.is_empty() {
            return Err(GatewayError::InvalidResponse(
                "no vnp_ResponseCode in the response".into(),
            ));
        }
        // A forged "no" only makes us skip a refund, so it needn't be signed. A forged "yes"
        // would have us mark a payment refunded that never was, so that one must be.
        if code != "00" {
            let message = field(&body, "vnp_Message");
            warn!(txn_ref = %request.txn_ref, %code, %message, "VNPay refused the refund");
            return Err(GatewayError::Rejected { code, message });
        }
        if !self.response_is_signed(&body) {
            return Err(GatewayError::InvalidResponse(
                "vnp_SecureHash does not match".into(),
            ));
        }

        let transaction_status = field(&body, "vnp_TransactionStatus");
        let message = field(&body, "vnp_Message");
        match transaction_status.as_str() {
            "00" => {
                let transaction_no = field(&body, "vnp_TransactionNo");
                info!(txn_ref = %request.txn_ref, %transaction_no, "VNPay refunded the payment");
                Ok(RefundReceipt {
                    transaction_no,
                    message,
                })
            }
            // with VNPay (05) or already sent on to the bank (06)
            "05" | "06" => Err(GatewayError::InProgress),
            other => Err(GatewayError::Rejected {
                code: other.to_string(),
                message,
            }),
        }
    }
}

/// What VNPay hashes for a refund request. `fields` are in VNPay's order, see `refund_body`.
fn refund_signing_data(fields: &[&str; 13]) -> String {
    fields.join("|")
}

fn response_signing_data(body: &Value) -> String {
    RESPONSE_SIGNED_FIELDS
        .iter()
        .map(|key| field(body, key))
        .collect::<Vec<_>>()
        .join("|")
}

/// A response field as text. VNPay sends some numbers as JSON numbers and some as strings,
/// and leaves out what it has no value for, which counts as an empty string when signing.
fn field(body: &Value, key: &str) -> String {
    match body.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn config() -> PaymentConfig {
        PaymentConfig {
            tmn_code: "TMN12345".into(),
            hash_secret: "secret".into(),
            payment_url: "https://pay.example/vpcpay.html".into(),
            return_url: "https://shop.example/return".into(),
            ipn_url: "https://shop.example/ipn".into(),
            default_locale: "vn".into(),
            order_type: "other".into(),
            qr_expiry_minutes: 15,
            api_url: "https://api.example/transaction".into(),
            server_ip: "10.0.0.7".into(),
            api_timeout_secs: 10,
        }
    }

    fn request() -> RefundRequest {
        RefundRequest {
            txn_ref: "123456ABCDEF".into(),
            amount: 150_000,
            transaction_no: Some("14000001".into()),
            transaction_date: Utc.with_ymd_and_hms(2026, 10, 8, 9, 30, 15).unwrap(),
            requested_by: "admin-1".into(),
            reason: "Hoan tien don hang 42".into(),
        }
    }

    #[test]
    fn refund_signing_data_follows_the_documented_field_order() {
        let data = refund_signing_data(&[
            "req1", "2.1.0", "refund", "TMN12345", "02", "123456ABCDEF", "15000000", "14000001",
            "20261008093015", "admin-1", "20261008100000", "10.0.0.7", "Hoan tien",
        ]);

        assert_eq!(
            data,
            "req1|2.1.0|refund|TMN12345|02|123456ABCDEF|15000000|14000001|20261008093015|admin-1|20261008100000|10.0.0.7|Hoan tien"
        );
    }

    #[test]
    fn response_signing_data_follows_the_documented_field_order_and_blanks_missing_fields() {
        let body = json!({
            "vnp_ResponseId": "r1",
            "vnp_Command": "refund",
            "vnp_ResponseCode": "00",
            "vnp_Message": "Refund Success",
            "vnp_TmnCode": "TMN12345",
            "vnp_TxnRef": "123456ABCDEF",
            "vnp_Amount": 15000000,
            "vnp_TransactionNo": 14000099,
            "vnp_TransactionType": "02",
            "vnp_TransactionStatus": "00",
            "vnp_OrderInfo": "Hoan tien",
            "vnp_SecureHash": "ignored"
        });

        // bank code and pay date are absent, so their slots are empty
        assert_eq!(
            response_signing_data(&body),
            "r1|refund|00|Refund Success|TMN12345|123456ABCDEF|15000000|||14000099|02|00|Hoan tien"
        );
    }

    #[test]
    fn field_reads_strings_and_numbers_and_defaults_to_empty() {
        let body = json!({ "a": "text", "b": 7, "c": null });

        assert_eq!(field(&body, "a"), "text");
        assert_eq!(field(&body, "b"), "7");
        assert_eq!(field(&body, "c"), "");
        assert_eq!(field(&body, "missing"), "");
    }

    #[test]
    fn refund_body_carries_amount_in_hundredths_and_a_signature_over_its_own_fields() {
        let gateway = VnPayGateway::new(&config()).unwrap();

        let body = gateway.refund_body(&request());

        assert_eq!(body["vnp_Amount"], 15_000_000);
        assert_eq!(body["vnp_TransactionType"], "02");
        assert_eq!(body["vnp_TransactionDate"], "20261008093015");
        assert_eq!(body["vnp_IpAddr"], "10.0.0.7");
        let expected = VnPayClient::sign(
            "secret",
            &refund_signing_data(&[
                body["vnp_RequestId"].as_str().unwrap(),
                "2.1.0",
                "refund",
                "TMN12345",
                "02",
                "123456ABCDEF",
                "15000000",
                "14000001",
                "20261008093015",
                "admin-1",
                body["vnp_CreateDate"].as_str().unwrap(),
                "10.0.0.7",
                "Hoan tien don hang 42",
            ]),
        );
        assert_eq!(body["vnp_SecureHash"], expected.as_str());
    }

    #[test]
    fn refund_body_leaves_out_transaction_no_when_there_is_none() {
        let gateway = VnPayGateway::new(&config()).unwrap();
        let mut req = request();
        req.transaction_no = None;

        let body = gateway.refund_body(&req);

        assert!(body.get("vnp_TransactionNo").is_none());
    }

    #[test]
    fn every_refund_gets_its_own_request_id() {
        let gateway = VnPayGateway::new(&config()).unwrap();

        let first = gateway.refund_body(&request());
        let second = gateway.refund_body(&request());

        assert_ne!(first["vnp_RequestId"], second["vnp_RequestId"]);
        assert!(first["vnp_RequestId"].as_str().unwrap().len() <= 32);
    }
}
