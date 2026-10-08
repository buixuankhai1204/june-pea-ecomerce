//! Integration tests for `VnPayGateway`: our reqwest adapter talking real HTTP to a fake VNPay
//! (`test_support::vnpay::VnPayStub`, a wiremock server). Nothing else is faked, so what's under
//! test is the part that only shows on the wire: the JSON we send, the signatures both ways,
//! and what we make of every way the other side can fail.
//!
//! No Docker needed, the "other service" is a socket in this process.

mod common;

use chrono::{TimeZone, Utc};
use common::test_config;
use payment::domain::{GatewayError, PaymentGateway, RefundRequest};
use payment::infrastructure::vnpay::VnPayGateway;
use std::time::Duration;
use test_support::vnpay::{refund_request_is_signed, Behaviour, VnPayStub, HASH_SECRET, TMN_CODE};
use test_support::wait::free_port;

fn refund_request() -> RefundRequest {
    RefundRequest {
        txn_ref: "123456ABCDEF".into(),
        amount: 150_000,
        transaction_no: Some("14000001".into()),
        transaction_date: Utc.with_ymd_and_hms(2026, 10, 8, 9, 30, 15).unwrap(),
        requested_by: "admin-1".into(),
        reason: "Hoan tien don hang 42".into(),
    }
}

async fn gateway_for(stub: &VnPayStub) -> VnPayGateway {
    VnPayGateway::new(&test_config(stub.api_url())).unwrap()
}

#[tokio::test]
async fn sends_a_signed_refund_request_and_returns_the_receipt() {
    let vnpay = VnPayStub::start().await;

    let receipt = gateway_for(&vnpay)
        .await
        .refund(refund_request())
        .await
        .unwrap();

    assert_eq!(receipt.transaction_no, "14000099");
    let sent = vnpay.refund_requests().await;
    assert_eq!(sent.len(), 1);
    let body = &sent[0];
    assert!(refund_request_is_signed(body, HASH_SECRET));
    assert_eq!(body["vnp_Command"], "refund");
    assert_eq!(body["vnp_Version"], "2.1.0");
    assert_eq!(body["vnp_TmnCode"], TMN_CODE);
    assert_eq!(body["vnp_TransactionType"], "02", "02 is a full refund");
    assert_eq!(body["vnp_TxnRef"], "123456ABCDEF");
    assert_eq!(body["vnp_Amount"], 15_000_000, "VNPay counts in hundredths");
    assert_eq!(body["vnp_TransactionNo"], "14000001");
    assert_eq!(body["vnp_TransactionDate"], "20261008093015");
    assert_eq!(body["vnp_CreateBy"], "admin-1");
    assert_eq!(body["vnp_OrderInfo"], "Hoan tien don hang 42");
    assert_eq!(body["vnp_IpAddr"], "127.0.0.1");
    assert_eq!(body["vnp_CreateDate"].as_str().unwrap().len(), 14);
}

#[tokio::test]
async fn a_payment_without_a_provider_transaction_no_can_still_be_refunded() {
    let vnpay = VnPayStub::start().await;
    let mut request = refund_request();
    request.transaction_no = None;

    gateway_for(&vnpay).await.refund(request).await.unwrap();

    // not in the JSON, and the stub (like VNPay) still verified the signature over an empty slot
    assert!(vnpay.refund_requests().await[0].get("vnp_TransactionNo").is_none());
}

#[tokio::test]
async fn a_refusal_comes_back_with_vnpays_code_and_message() {
    let vnpay = VnPayStub::start().await;
    vnpay
        .behave(Behaviour::Refuses {
            code: "94",
            message: "Duplicate request",
        })
        .await;

    let err = gateway_for(&vnpay)
        .await
        .refund(refund_request())
        .await
        .unwrap_err();

    assert_eq!(
        err,
        GatewayError::Rejected {
            code: "94".into(),
            message: "Duplicate request".into()
        }
    );
}

#[tokio::test]
async fn a_wrong_hash_secret_is_refused_by_vnpay_as_an_invalid_signature() {
    let vnpay = VnPayStub::start().await;
    let mut config = test_config(vnpay.api_url());
    config.hash_secret = "someone-elses-secret".into();

    let err = VnPayGateway::new(&config)
        .unwrap()
        .refund(refund_request())
        .await
        .unwrap_err();

    assert!(matches!(err, GatewayError::Rejected { ref code, .. } if code == "97"), "{err:?}");
}

#[tokio::test]
async fn a_refund_that_is_still_with_the_bank_is_in_progress() {
    let vnpay = VnPayStub::start().await;
    vnpay.behave(Behaviour::StillProcessing).await;

    let err = gateway_for(&vnpay)
        .await
        .refund(refund_request())
        .await
        .unwrap_err();

    assert_eq!(err, GatewayError::InProgress);
}

#[tokio::test]
async fn an_http_error_from_vnpay_is_unavailable() {
    let vnpay = VnPayStub::start().await;
    vnpay.behave(Behaviour::Down).await;

    let err = gateway_for(&vnpay)
        .await
        .refund(refund_request())
        .await
        .unwrap_err();

    assert!(matches!(&err, GatewayError::Unavailable(m) if m.contains("500")), "{err:?}");
}

#[tokio::test]
async fn nobody_listening_is_unavailable() {
    let nobody_home = format!(
        "http://127.0.0.1:{}/merchant_webapi/api/transaction",
        free_port()
    );

    let err = VnPayGateway::new(&test_config(nobody_home))
        .unwrap()
        .refund(refund_request())
        .await
        .unwrap_err();

    assert!(matches!(err, GatewayError::Unavailable(_)), "{err:?}");
}

#[tokio::test]
async fn an_answer_that_takes_too_long_is_unavailable() {
    let vnpay = VnPayStub::start().await;
    vnpay.behave(Behaviour::Slow(Duration::from_secs(2))).await;
    let gateway =
        VnPayGateway::with_timeout(&test_config(vnpay.api_url()), Duration::from_millis(200))
            .unwrap();

    let err = gateway.refund(refund_request()).await.unwrap_err();

    assert!(matches!(&err, GatewayError::Unavailable(m) if m.contains("timed out")), "{err:?}");
}

#[tokio::test]
async fn an_answer_that_is_not_json_is_not_trusted() {
    let vnpay = VnPayStub::start().await;
    vnpay.behave(Behaviour::Garbled).await;

    let err = gateway_for(&vnpay)
        .await
        .refund(refund_request())
        .await
        .unwrap_err();

    assert!(matches!(err, GatewayError::InvalidResponse(_)), "{err:?}");
}

#[tokio::test]
async fn a_success_signed_with_the_wrong_secret_is_not_trusted() {
    let vnpay = VnPayStub::start().await;
    vnpay.behave(Behaviour::ForgedSuccess).await;

    let err = gateway_for(&vnpay)
        .await
        .refund(refund_request())
        .await
        .unwrap_err();

    assert!(matches!(&err, GatewayError::InvalidResponse(m) if m.contains("SecureHash")), "{err:?}");
}
