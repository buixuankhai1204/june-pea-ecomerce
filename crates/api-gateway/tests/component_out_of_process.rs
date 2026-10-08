//! Component tests that start the real `api-gateway` binary as a child process, configured
//! through env vars like it would be in prod, and only talk to it from the outside.
//!
//! Real: the binary (`main.rs` wiring, env config, migrations, the JWT middleware), Postgres,
//! Redis. Faked: VNPay, as a wiremock server the gateway is pointed at with `VNPAY_API_URL`.
//!
//! This covers what the in-process tests can't: that everything is wired together and the
//! checkout journey works through the real auth layer and real SQL. It needs Docker (or
//! `TEST_DATABASE_URL` and `TEST_REDIS_URL`), so it is ignored by default:
//!
//! ```text
//! cargo test -p api-gateway --test component_out_of_process -- --ignored
//! ```
//!
//! Building the gateway needs a migrated database at compile time (`analytics` uses
//! `sqlx::query!`), so `DATABASE_URL` must be set for that too.

mod common;

use common::*;
use serde_json::{json, Value};
use test_support::vnpay::{paid_ipn_query, signed_ipn_query, Behaviour, HASH_SECRET};

impl Gateway {
    async fn json(&self, req: reqwest::RequestBuilder) -> (u16, Value) {
        let res = req.send().await.unwrap();
        (res.status().as_u16(), res.json().await.unwrap_or(Value::Null))
    }

    /// places an order for one seeded laptop and returns its id
    async fn place_order(&self, token: &str, customer: uuid::Uuid) -> String {
        let (status, body) = self
            .json(self.http.post(self.url("/ordering/orders")).bearer_auth(token).json(&json!({
                "customer_id": customer,
                "items": [{ "variant_id": SEEDED_VARIANT, "quantity": 1, "unit_price": SEEDED_VARIANT_PRICE }]
            })))
            .await;
        assert_eq!(status, 200, "{body}");
        body["order_id"].as_str().unwrap().to_string()
    }

    /// a customer's checkout up to and including VNPay's IPN
    async fn checkout_and_pay(&self, token: &str, customer: uuid::Uuid) -> (String, String) {
        let order_id = self.place_order(token, customer).await;
        let (status, qr) = self
            .json(self.http.post(self.url("/payment/vnpay/qr")).bearer_auth(token).json(&json!({ "order_id": order_id })))
            .await;
        assert_eq!(status, 200, "{qr}");
        assert_eq!(qr["status"], "Pending");
        let txn_ref = qr["txn_ref"].as_str().unwrap().to_string();

        // VNPay's callback carries no token
        let ipn_url = format!("{}?{}", self.url("/payment/vnpay/ipn"), paid_ipn_query(HASH_SECRET, &txn_ref, SEEDED_VARIANT_PRICE));
        let (_, ipn) = self.json(self.http.get(ipn_url)).await;
        assert_eq!(ipn["rsp_code"], "00", "{ipn}");
        (order_id, txn_ref)
    }

    async fn payment_status(&self, token: &str, order_id: &str) -> String {
        let (_, body) = self
            .json(self.http.get(self.url(&format!("/payment/orders/{order_id}"))).bearer_auth(token))
            .await;
        body["status"].as_str().unwrap().to_string()
    }

    async fn order_status(&self, token: &str, order_id: &str) -> String {
        let (_, body) = self
            .json(self.http.get(self.url(&format!("/ordering/orders/{order_id}"))).bearer_auth(token))
            .await;
        body["status"].as_str().unwrap().to_string()
    }
}

#[tokio::test]
#[ignore = "requires Docker (or TEST_DATABASE_URL and TEST_REDIS_URL)"]
async fn starts_migrated_and_serves_the_seeded_catalog() {
    let gateway = Gateway::start().await;

    let (status, body) = gateway.json(gateway.http.get(gateway.url("/catalog/products"))).await;

    assert_eq!(status, 200);
    assert!(body.to_string().contains("Pro Laptop"), "{body}");
}

#[tokio::test]
#[ignore = "requires Docker (or TEST_DATABASE_URL and TEST_REDIS_URL)"]
async fn payment_routes_need_a_login_but_the_ipn_does_not() {
    let gateway = Gateway::start().await;

    let qr = gateway.http.post(gateway.url("/payment/vnpay/qr")).json(&json!({ "order_id": uuid::Uuid::nil() })).send().await.unwrap();
    let forged = signed_ipn_query("someone-elses-secret", &[("vnp_TxnRef", "x"), ("vnp_Amount", "100")]);
    let ipn = gateway.http.get(format!("{}?{forged}", gateway.url("/payment/vnpay/ipn"))).send().await.unwrap();

    assert_eq!(qr.status(), 401);
    assert_eq!(ipn.status(), 200);
    assert_eq!(ipn.json::<Value>().await.unwrap()["rsp_code"], "97");
}

#[tokio::test]
#[ignore = "requires Docker (or TEST_DATABASE_URL and TEST_REDIS_URL)"]
async fn a_customer_pays_and_an_admin_refunds() {
    let gateway = Gateway::start().await;
    let (customer, customer_token) = gateway.user("customer").await;
    let (admin, admin_token) = gateway.user("admin").await;
    let (order_id, txn_ref) = gateway.checkout_and_pay(&customer_token, customer).await;
    assert_eq!(gateway.payment_status(&customer_token, &order_id).await, "Paid");
    assert_eq!(gateway.order_status(&customer_token, &order_id).await, "Completed");

    // a customer's token is not enough to move money
    let refund_url = gateway.url(&format!("/payment/orders/{order_id}/refund"));
    let as_customer = gateway.http.post(&refund_url).bearer_auth(&customer_token).send().await.unwrap();
    assert_eq!(as_customer.status(), 403);
    assert!(gateway.vnpay.refund_requests().await.is_empty());

    let as_admin = gateway.http.post(&refund_url).bearer_auth(&admin_token).send().await.unwrap();

    assert_eq!(as_admin.status(), 200);
    let sent = gateway.vnpay.refund_requests().await;
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["vnp_TxnRef"], txn_ref.as_str());
    assert_eq!(sent[0]["vnp_CreateBy"], admin.to_string());
    assert_eq!(gateway.payment_status(&admin_token, &order_id).await, "Refunded");
}

#[tokio::test]
#[ignore = "requires Docker (or TEST_DATABASE_URL and TEST_REDIS_URL)"]
async fn a_refund_while_vnpay_is_down_is_a_502_and_the_payment_stays_paid() {
    let gateway = Gateway::start().await;
    let (customer, customer_token) = gateway.user("customer").await;
    let (_, admin_token) = gateway.user("admin").await;
    let (order_id, _) = gateway.checkout_and_pay(&customer_token, customer).await;
    gateway.vnpay.behave(Behaviour::Down).await;

    let res = gateway
        .http
        .post(gateway.url(&format!("/payment/orders/{order_id}/refund")))
        .bearer_auth(&admin_token)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 502);
    assert_eq!(gateway.payment_status(&admin_token, &order_id).await, "Paid");
}
