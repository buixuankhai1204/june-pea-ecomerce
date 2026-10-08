//! End-to-end test against a deployed environment (the real gateway, database and cache),
//! through the public API only. It is slow and needs infrastructure, so it is ignored by
//! default and meant for a late pipeline stage:
//!
//! ```text
//! E2E_BASE_URL=https://staging.example.com cargo test -p api-gateway --test e2e -- --ignored
//! ```
//!
//! VNPay is real here, so the journey stops where a person would scan the QR code: it checks
//! that checkout produces a pending payment pointing at VNPay. Set `E2E_VARIANT_ID` and
//! `E2E_UNIT_PRICE` if the environment's catalog isn't the seeded one.
//!
//! Keep these few, one per critical user journey. Edge cases go in the faster tests.

use serde_json::{json, Value};
use uuid::Uuid;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::test]
#[ignore = "needs a deployed environment: set E2E_BASE_URL"]
async fn a_new_customer_can_register_order_and_get_a_payment_to_scan() {
    let base = std::env::var("E2E_BASE_URL")
        .expect("set E2E_BASE_URL to the deployed environment, e.g. https://staging.example.com")
        .trim_end_matches('/')
        .to_string();
    let variant = env_or("E2E_VARIANT_ID", "cccccccc-cccc-cccc-cccc-cccccccccccc");
    let unit_price: i64 = env_or("E2E_UNIT_PRICE", "140000").parse().unwrap();
    let http = reqwest::Client::new();
    let api = |path: &str| format!("{base}/api/v1{path}");

    // register and log in
    let email = format!("e2e-{}@example.test", Uuid::new_v4());
    let registered = http
        .post(api("/auth/register"))
        .json(&json!({ "email": email, "password": "pw-123456", "password_confirm": "pw-123456" }))
        .send()
        .await
        .unwrap();
    assert!(registered.status().is_success(), "register failed: {}", registered.status());
    let login: Value = http
        .post(api("/auth/login"))
        .json(&json!({ "email": email, "password": "pw-123456" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = login["token"].as_str().expect("login returned a token");

    // order
    let placed: Value = http
        .post(api("/ordering/orders"))
        .bearer_auth(token)
        .json(&json!({ "items": [{ "variant_id": variant, "quantity": 1, "unit_price": unit_price }] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let order_id = placed["order_id"].as_str().expect("order has an id").to_string();

    // checkout
    let qr: Value = http
        .post(api("/payment/vnpay/qr"))
        .bearer_auth(token)
        .json(&json!({ "order_id": order_id }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(qr["status"], "Pending");
    assert_eq!(qr["amount"], unit_price);
    assert!(qr["payment_url"].as_str().unwrap().contains("vnp_SecureHash="));
    assert!(qr["qr_svg"].as_str().unwrap().contains("<svg"));

    // and the payment can be looked up again
    let status: Value = http
        .get(api(&format!("/payment/orders/{order_id}")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["txn_ref"], qr["txn_ref"]);
}
