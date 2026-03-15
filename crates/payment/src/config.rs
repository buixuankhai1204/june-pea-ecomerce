use anyhow::Context;
use std::env;

#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub tmn_code: String,
    pub hash_secret: String,
    pub payment_url: String,
    pub return_url: String,
    pub ipn_url: String,
    pub default_locale: String,
    pub order_type: String,
    pub qr_expiry_minutes: i64,
}

impl PaymentConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            tmn_code: env::var("VNPAY_TMNCODE").context("VNPAY_TMNCODE is missing")?,
            hash_secret: env::var("VNPAY_HASH_SECRET").context("VNPAY_HASH_SECRET is missing")?,
            payment_url: env::var("VNPAY_PAYMENT_URL").context("VNPAY_PAYMENT_URL is missing")?,
            return_url: env::var("VNPAY_RETURN_URL").context("VNPAY_RETURN_URL is missing")?,
            ipn_url: env::var("VNPAY_IPN_URL").context("VNPAY_IPN_URL is missing")?,
            default_locale: env::var("VNPAY_DEFAULT_LOCALE").unwrap_or_else(|_| "vn".into()),
            order_type: env::var("VNPAY_ORDER_TYPE").unwrap_or_else(|_| "other".into()),
            qr_expiry_minutes: env::var("VNPAY_QR_EXPIRE_MINUTES")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(15),
        })
    }
}
