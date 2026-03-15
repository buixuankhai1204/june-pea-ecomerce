use crate::config::PaymentConfig;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use sha2::Sha512;
use std::collections::BTreeMap;
use tracing::debug;

pub struct VnPayClient {
    config: PaymentConfig,
}

#[derive(Debug, Clone)]
pub struct VnPayPaymentRequest {
    pub txn_ref: String,
    pub amount: i64,
    pub order_info: String,
    pub client_ip: Option<String>,
    pub expire_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VnPayPaymentResponse {
    pub payment_url: String,
    pub secure_hash: String,
}

impl VnPayClient {
    pub fn new(config: PaymentConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &PaymentConfig {
        &self.config
    }

    pub fn build_payment_url(
        &self,
        req: &VnPayPaymentRequest,
    ) -> anyhow::Result<VnPayPaymentResponse> {
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("vnp_Version".into(), "2.1.0".to_string());
        params.insert("vnp_Command".into(), "pay".to_string());
        params.insert("vnp_TmnCode".into(), self.config.tmn_code.clone());
        params.insert("vnp_Amount".into(), format!("{}", req.amount * 100));
        params.insert("vnp_CurrCode".into(), "VND".to_string());
        params.insert("vnp_TxnRef".into(), req.txn_ref.clone());
        params.insert("vnp_OrderInfo".into(), req.order_info.clone());
        params.insert("vnp_OrderType".into(), self.config.order_type.clone());
        params.insert("vnp_Locale".into(), self.config.default_locale.clone());
        params.insert("vnp_ReturnUrl".into(), self.config.return_url.clone());
        params.insert(
            "vnp_IpAddr".into(),
            req.client_ip
                .clone()
                .unwrap_or_else(|| "0.0.0.0".to_string()),
        );
        params.insert(
            "vnp_CreateDate".into(),
            Utc::now().format("%Y%m%d%H%M%S").to_string(),
        );
        params.insert(
            "vnp_ExpireDate".into(),
            req.expire_at.format("%Y%m%d%H%M%S").to_string(),
        );
        params.insert("vnp_BankCode".into(), "VNPAYQR".to_string());

        let query_string = Self::build_query_string(&params);
        let secure_hash = Self::sign(&self.config.hash_secret, &query_string);
        let payment_url = format!(
            "{}?{}&vnp_SecureHash={}",
            self.config.payment_url, query_string, secure_hash
        );

        Ok(VnPayPaymentResponse {
            payment_url,
            secure_hash,
        })
    }

    pub fn verify_signature(&self, params: &BTreeMap<String, String>) -> bool {
        if let Some(client_hash) = params.get("vnp_SecureHash") {
            let mut filtered = params.clone();
            filtered.remove("vnp_SecureHash");
            filtered.remove("vnp_SecureHashType");
            let query = Self::build_query_string(&filtered);
            let expected_hash = Self::sign(&self.config.hash_secret, &query);
            let matched = expected_hash.eq_ignore_ascii_case(client_hash);
            if !matched {
                debug!(
                    "VNPay signature mismatch: expected {}, got {}",
                    expected_hash, client_hash
                );
            }
            matched
        } else {
            false
        }
    }

    pub fn random_txn_ref() -> String {
        let suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        format!("{}{}", Utc::now().format("%H%M%S"), suffix)
    }

    fn build_query_string(params: &BTreeMap<String, String>) -> String {
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    fn sign(secret: &str, data: &str) -> String {
        let mut mac = Hmac::<Sha512>::new_from_slice(secret.as_bytes()).expect("valid key");
        mac.update(data.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_is_deterministic() {
        let hash = VnPayClient::sign("secret", "foo=bar");
        assert_eq!(hash.len(), 128);
        let hash2 = VnPayClient::sign("secret", "foo=bar");
        assert_eq!(hash, hash2);
    }
}
