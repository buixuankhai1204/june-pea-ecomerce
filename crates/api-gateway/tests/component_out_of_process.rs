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

// ── who may reach what ───────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires Docker (or TEST_DATABASE_URL and TEST_REDIS_URL)"]
async fn the_catalog_is_open_to_read_but_only_staff_can_change_it() {
    let gateway = Gateway::start().await;
    let (_, customer_token) = gateway.user("customer").await;
    let (_, staff_token) = gateway.user("staff").await;
    let body = json!({ "name": "Hats", "slug": "hats" });
    let create = |token: Option<&str>| {
        let req = gateway.http.post(gateway.url("/catalog/categories")).json(&body);
        match token {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    };

    let (read, _) = gateway.json(gateway.http.get(gateway.url("/catalog/categories"))).await;
    let (anonymous, _) = gateway.json(create(None)).await;
    let (customer, _) = gateway.json(create(Some(&customer_token))).await;
    let (bad_token, _) = gateway.json(create(Some("not-a-jwt"))).await;
    let (staff, _) = gateway.json(create(Some(&staff_token))).await;

    assert_eq!((read, anonymous, customer, bad_token, staff), (200, 401, 403, 401, 200));
}

#[tokio::test]
#[ignore = "requires Docker (or TEST_DATABASE_URL and TEST_REDIS_URL)"]
async fn user_administration_is_not_reachable_without_an_admin_login() {
    let gateway = Gateway::start().await;
    let (_, customer_token) = gateway.user("customer").await;
    let (_, staff_token) = gateway.user("staff").await;
    let (_, admin_token) = gateway.user("admin").await;

    // the public /auth mount only has register and login
    for path in ["/auth/users", "/auth/staff", "/auth/memberships", "/auth/me"] {
        let (status, _) = gateway.json(gateway.http.get(gateway.url(path))).await;
        assert_eq!(status, 404, "GET {path}");
    }
    let users = gateway.url("/identity/users");
    let (anonymous, _) = gateway.json(gateway.http.get(&users)).await;
    let (as_customer, _) = gateway.json(gateway.http.get(&users).bearer_auth(&customer_token)).await;
    let (as_staff, _) = gateway.json(gateway.http.get(&users).bearer_auth(&staff_token)).await;
    let (as_admin, list) = gateway.json(gateway.http.get(&users).bearer_auth(&admin_token)).await;

    assert_eq!((anonymous, as_customer, as_staff, as_admin), (401, 403, 403, 200));
    assert!(list.as_array().unwrap().len() >= 3);
    assert!(!list.to_string().contains("argon2"), "the user list leaked a password hash: {list}");
    assert!(!list.to_string().contains("password"), "{list}");
}

#[tokio::test]
#[ignore = "requires Docker (or TEST_DATABASE_URL and TEST_REDIS_URL)"]
async fn a_customer_cannot_reach_other_customers_orders_or_the_staff_routes() {
    let gateway = Gateway::start().await;
    let (alice, alice_token) = gateway.user("customer").await;
    let (bob, bob_token) = gateway.user("customer").await;
    let (_, staff_token) = gateway.user("staff").await;
    let order_id = gateway.place_order(&alice_token, alice).await;
    let order = gateway.url(&format!("/ordering/orders/{order_id}"));

    // bob cannot read, cancel or list alice's orders, and cannot order in her name
    let (read, _) = gateway.json(gateway.http.get(&order).bearer_auth(&bob_token)).await;
    let (cancel, _) = gateway.json(gateway.http.delete(&order).bearer_auth(&bob_token)).await;
    let (list, _) = gateway.json(gateway.http.get(gateway.url(&format!("/ordering/orders/customer/{alice}"))).bearer_auth(&bob_token)).await;
    let (forged, _) = gateway
        .json(gateway.http.post(gateway.url("/ordering/orders")).bearer_auth(&bob_token).json(&json!({
            "customer_id": alice,
            "items": [{ "variant_id": SEEDED_VARIANT, "quantity": 1, "unit_price": SEEDED_VARIANT_PRICE }]
        })))
        .await;
    assert_eq!((read, cancel, list, forged), (403, 403, 403, 403));

    // nor the staff routes, whose answers are 403 for him and 200 for staff
    let (all_as_bob, _) = gateway.json(gateway.http.get(gateway.url("/ordering/orders")).bearer_auth(&bob_token)).await;
    let (all_as_staff, _) = gateway.json(gateway.http.get(gateway.url("/ordering/orders")).bearer_auth(&staff_token)).await;
    let (stock_as_bob, _) = gateway.json(gateway.http.get(gateway.url("/inventory/list-all")).bearer_auth(&bob_token)).await;
    let (dashboard_as_bob, _) = gateway.json(gateway.http.get(gateway.url("/analytics/dashboard-summary")).bearer_auth(&bob_token)).await;
    assert_eq!((all_as_bob, all_as_staff, stock_as_bob, dashboard_as_bob), (403, 200, 403, 403));

    // and alice's order is untouched, still hers to read
    assert_eq!(gateway.order_status(&alice_token, &order_id).await, "Pending");
    let _ = bob;
}
