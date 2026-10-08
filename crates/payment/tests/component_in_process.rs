//! Component tests with the payment routes running inside the test process.
//!
//! Wired like `api-gateway/src/main.rs` (IPN route public, the rest behind auth) except the
//! database is replaced by in-memory repositories and the JWT middleware by a header
//! (`x-test-role`). Requests go over a real socket (port 0, so any free port) and the refund
//! goes out over real HTTP to a fake VNPay, so routing, JSON, status codes, the reqwest gateway
//! and both signatures all get exercised. Every test gets its own app and its own VNPay.
//!
//! What this can't show: `main.rs` wiring, env var config, the real JWT middleware and real
//! SQL. `api-gateway/tests/component_out_of_process.rs` and `integration_postgres.rs` cover those.

mod common;

use common::*;
use ordering::domain::model::OrderStatus;
use payment::domain::PaymentStatus;
use serde_json::json;
use test_support::vnpay::{paid_ipn_query, signed_ipn_query, Behaviour, HASH_SECRET};
use uuid::Uuid;

// asking for a QR code

#[tokio::test]
async fn qr_request_returns_a_pending_payment_with_a_signed_checkout_url() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;

    let reply = app.request_qr(order_id, "customer").await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body["status"], "Pending");
    assert_eq!(reply.body["amount"], 150_000);
    assert_eq!(reply.body["order_id"], order_id.to_string());
    let url = reply.body["payment_url"].as_str().unwrap();
    assert!(url.starts_with("https://pay.example/vpcpay.html?"), "{url}");
    assert!(url.contains("vnp_Amount=15000000"), "VNPay counts in hundredths: {url}");
    assert!(url.contains("vnp_SecureHash="));
    assert!(reply.body["qr_svg"].as_str().unwrap().contains("<svg"));
}

#[tokio::test]
async fn asking_again_for_the_same_order_returns_the_same_payment() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;

    let first = app.request_qr(order_id, "customer").await;
    let second = app.request_qr(order_id, "customer").await;

    assert_eq!(second.body["payment_id"], first.body["payment_id"]);
    assert_eq!(second.body["txn_ref"], first.body["txn_ref"]);
}

#[tokio::test]
async fn qr_request_for_an_unknown_order_is_404() {
    let app = spawn_app().await;

    let reply = app.request_qr(Uuid::new_v4(), "customer").await;

    assert_eq!(reply.status, 404);
}

#[tokio::test]
async fn qr_request_for_an_order_that_is_not_pending_is_409() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;
    app.orders.set_status(order_id, OrderStatus::Cancelled);

    let reply = app.request_qr(order_id, "customer").await;

    assert_eq!(reply.status, 409);
}

#[tokio::test]
async fn the_routes_behind_auth_need_a_login() {
    let app = spawn_app().await;

    let res = app
        .http
        .post(format!("{}/api/v1/payment/vnpay/qr", app.base))
        .json(&json!({ "order_id": Uuid::new_v4() }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

// VNPay telling us about the payment (IPN)

#[tokio::test]
async fn a_signed_ipn_marks_the_payment_paid_and_completes_the_order() {
    let app = spawn_app().await;

    let (order_id, _) = app.paid_order(150_000).await;

    let status = app.payment_status(order_id, "customer").await;
    assert_eq!(status.body["status"], "Paid");
    assert_eq!(app.orders.status_of(order_id), OrderStatus::Completed);
}

#[tokio::test]
async fn an_ipn_with_a_bad_signature_is_refused_and_changes_nothing() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;
    let qr = app.request_qr(order_id, "customer").await;
    let txn_ref = qr.body["txn_ref"].as_str().unwrap();

    let forged = paid_ipn_query("not-the-merchants-secret", txn_ref, 150_000);
    let reply = app.ipn(&forged).await;

    assert_eq!(reply.body["rsp_code"], "97");
    assert_eq!(
        app.payment_status(order_id, "customer").await.body["status"],
        "Pending"
    );
    assert_eq!(app.orders.status_of(order_id), OrderStatus::Pending);
}

#[tokio::test]
async fn an_ipn_for_the_wrong_amount_is_refused_and_changes_nothing() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;
    let qr = app.request_qr(order_id, "customer").await;
    let txn_ref = qr.body["txn_ref"].as_str().unwrap();

    let reply = app.ipn(&paid_ipn_query(HASH_SECRET, txn_ref, 1_000)).await;

    assert_eq!(reply.body["rsp_code"], "04");
    assert_eq!(app.orders.status_of(order_id), OrderStatus::Pending);
}

#[tokio::test]
async fn a_declined_payment_is_recorded_as_failed_and_the_order_stays_pending() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;
    let qr = app.request_qr(order_id, "customer").await;
    let txn_ref = qr.body["txn_ref"].as_str().unwrap();

    let declined = signed_ipn_query(
        HASH_SECRET,
        &[
            ("vnp_TxnRef", txn_ref),
            ("vnp_Amount", "15000000"),
            ("vnp_ResponseCode", "24"),
            ("vnp_TransactionStatus", "02"),
        ],
    );
    let reply = app.ipn(&declined).await;

    assert_eq!(reply.body["rsp_code"], "24");
    assert_eq!(
        app.payment_status(order_id, "customer").await.body["status"],
        "Failed"
    );
    assert_eq!(app.orders.status_of(order_id), OrderStatus::Pending);
}

#[tokio::test]
async fn an_ipn_delivered_twice_is_acknowledged_both_times_and_changes_nothing_the_second_time() {
    let app = spawn_app().await;
    let (order_id, txn_ref) = app.paid_order(150_000).await;
    let paid_at = app.payments.latest_for_order(order_id).unwrap().paid_at;

    let again = app.ipn(&paid_ipn_query(HASH_SECRET, &txn_ref, 150_000)).await;

    assert_eq!(again.body["rsp_code"], "00");
    assert_eq!(app.payments.latest_for_order(order_id).unwrap().paid_at, paid_at);
}

// refunds

#[tokio::test]
async fn an_admin_refund_goes_to_vnpay_and_then_shows_on_the_payment() {
    let app = spawn_app().await;
    let (order_id, txn_ref) = app.paid_order(150_000).await;

    let reply = app.refund(order_id, "admin").await;

    assert_eq!(reply.status, 200);
    let sent = app.vnpay.refund_requests().await;
    assert_eq!(sent.len(), 1, "exactly one call to VNPay");
    assert_eq!(sent[0]["vnp_TxnRef"], txn_ref.as_str());
    assert_eq!(sent[0]["vnp_Amount"], 15_000_000);
    assert_eq!(sent[0]["vnp_TransactionNo"], "14000001", "the number VNPay gave in the IPN");
    assert_eq!(sent[0]["vnp_CreateBy"], ADMIN_ID.to_string());
    let status = app.payment_status(order_id, "admin").await;
    assert_eq!(status.body["status"], "Refunded");
    assert_eq!(app.payments.latest_for_order(order_id).unwrap().status, PaymentStatus::Refunded);
}

#[tokio::test]
async fn the_refund_quotes_the_date_the_payment_url_was_created_with() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;
    let qr = app.request_qr(order_id, "customer").await;
    let url = qr.body["payment_url"].as_str().unwrap().to_string();
    let txn_ref = qr.body["txn_ref"].as_str().unwrap();
    app.ipn(&paid_ipn_query(HASH_SECRET, txn_ref, 150_000)).await;

    app.refund(order_id, "admin").await;

    // VNPay finds the payment by TxnRef and TransactionDate, so the date has to be the
    // vnp_CreateDate we put in the payment URL, to the second
    let create_date = url
        .split(['?', '&'])
        .find_map(|p| p.strip_prefix("vnp_CreateDate="))
        .unwrap();
    let sent = app.vnpay.refund_requests().await;
    assert_eq!(sent[0]["vnp_TransactionDate"], create_date);
}

#[tokio::test]
async fn customers_cannot_refund_and_vnpay_is_not_called() {
    let app = spawn_app().await;
    let (order_id, _) = app.paid_order(150_000).await;

    let reply = app.refund(order_id, "customer").await;

    assert_eq!(reply.status, 403);
    assert!(app.vnpay.refund_requests().await.is_empty());
    assert_eq!(app.payments.latest_for_order(order_id).unwrap().status, PaymentStatus::Paid);
}

#[tokio::test]
async fn refunding_a_payment_that_was_never_paid_is_400_and_vnpay_is_not_called() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;
    app.request_qr(order_id, "customer").await;

    let reply = app.refund(order_id, "admin").await;

    assert_eq!(reply.status, 400);
    assert!(app.vnpay.refund_requests().await.is_empty());
}

#[tokio::test]
async fn refunding_an_order_without_a_payment_is_404() {
    let app = spawn_app().await;
    let order_id = app.seed_order(150_000).await;

    let reply = app.refund(order_id, "admin").await;

    assert_eq!(reply.status, 404);
}

#[tokio::test]
async fn a_second_refund_is_turned_away_without_asking_vnpay_again() {
    let app = spawn_app().await;
    let (order_id, _) = app.paid_order(150_000).await;
    assert_eq!(app.refund(order_id, "admin").await.status, 200);

    let again = app.refund(order_id, "admin").await;

    assert_eq!(again.status, 400);
    assert_eq!(app.vnpay.refund_requests().await.len(), 1);
}

#[tokio::test]
async fn a_refund_vnpay_refuses_is_409_and_the_payment_stays_paid() {
    let app = spawn_app().await;
    let (order_id, _) = app.paid_order(150_000).await;
    app.vnpay
        .behave(Behaviour::Refuses {
            code: "91",
            message: "Transaction not found",
        })
        .await;

    let reply = app.refund(order_id, "admin").await;

    assert_eq!(reply.status, 409);
    assert!(reply.body["error"].as_str().unwrap().contains("91"));
    assert_eq!(app.payments.latest_for_order(order_id).unwrap().status, PaymentStatus::Paid);
}

#[tokio::test]
async fn a_refund_while_vnpay_is_down_is_502_and_the_payment_stays_paid() {
    let app = spawn_app().await;
    let (order_id, _) = app.paid_order(150_000).await;
    app.vnpay.behave(Behaviour::Down).await;

    let reply = app.refund(order_id, "admin").await;

    assert_eq!(reply.status, 502);
    assert_eq!(app.payments.latest_for_order(order_id).unwrap().status, PaymentStatus::Paid);

    // and once VNPay is back the same request goes through
    app.vnpay.behave(Behaviour::Refunds).await;
    assert_eq!(app.refund(order_id, "admin").await.status, 200);
}

#[tokio::test]
async fn a_refund_vnpay_has_not_finished_is_409_and_the_payment_stays_paid() {
    let app = spawn_app().await;
    let (order_id, _) = app.paid_order(150_000).await;
    app.vnpay.behave(Behaviour::StillProcessing).await;

    let reply = app.refund(order_id, "admin").await;

    assert_eq!(reply.status, 409);
    assert_eq!(app.payments.latest_for_order(order_id).unwrap().status, PaymentStatus::Paid);
}

#[tokio::test]
async fn a_forged_success_from_vnpay_does_not_refund_anything() {
    let app = spawn_app().await;
    let (order_id, _) = app.paid_order(150_000).await;
    app.vnpay.behave(Behaviour::ForgedSuccess).await;

    let reply = app.refund(order_id, "admin").await;

    assert_eq!(reply.status, 502);
    assert_eq!(app.payments.latest_for_order(order_id).unwrap().status, PaymentStatus::Paid);
}
