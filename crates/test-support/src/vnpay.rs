//! A stand-in for VNPay, in both directions.
//!
//! * VNPay calls us (the IPN callback): `signed_ipn_query` builds what VNPay would send.
//! * We call VNPay (refund): `VnPayStub` is a wiremock server that answers like VNPay's merchant
//!   API. It checks the request signature the way VNPay does and answers `97` if it is wrong,
//!   so a mistake in our signing fails a test instead of passing quietly.
//!
//! Everything here is written from VNPay's documentation, with its own HMAC code and its own
//! field lists, and shares nothing with the code under test.

use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha512;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

pub const TMN_CODE: &str = "TESTTMN1";
pub const HASH_SECRET: &str = "test-hash-secret-0123456789abcdef";
pub const API_PATH: &str = "/merchant_webapi/api/transaction";

/// The refund request fields VNPay signs, in the order it joins them.
const REQUEST_SIGNED_FIELDS: [&str; 13] = [
    "vnp_RequestId",
    "vnp_Version",
    "vnp_Command",
    "vnp_TmnCode",
    "vnp_TransactionType",
    "vnp_TxnRef",
    "vnp_Amount",
    "vnp_TransactionNo",
    "vnp_TransactionDate",
    "vnp_CreateBy",
    "vnp_CreateDate",
    "vnp_IpAddr",
    "vnp_OrderInfo",
];

/// The refund response fields VNPay signs, in the order it joins them.
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

fn hmac_sha512(secret: &str, data: &str) -> String {
    let mut mac = Hmac::<Sha512>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn text(body: &Value, key: &str) -> String {
    match body.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn signing_data(body: &Value, fields: &[&str]) -> String {
    fields
        .iter()
        .map(|k| text(body, k))
        .collect::<Vec<_>>()
        .join("|")
}

// VNPay -> us

/// The query string of an IPN callback: the params sorted, values URL-encoded, signed with
/// the same secret as everything else.
pub fn signed_ipn_query(secret: &str, params: &[(&str, &str)]) -> String {
    let mut sorted = params.to_vec();
    sorted.sort();
    let query = sorted
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let hash = hmac_sha512(secret, &query);
    format!("{query}&vnp_SecureHashType=HmacSHA512&vnp_SecureHash={hash}")
}

/// A successful payment as VNPay reports it. `amount_vnd` is in dong, VNPay sends hundredths.
pub fn paid_ipn_query(secret: &str, txn_ref: &str, amount_vnd: i64) -> String {
    signed_ipn_query(
        secret,
        &[
            ("vnp_TxnRef", txn_ref),
            ("vnp_Amount", &(amount_vnd * 100).to_string()),
            ("vnp_ResponseCode", "00"),
            ("vnp_TransactionStatus", "00"),
            ("vnp_TransactionNo", "14000001"),
            ("vnp_BankCode", "NCB"),
            ("vnp_OrderInfo", "Order"),
        ],
    )
}

// us -> VNPay

/// Does the refund request carry a signature that matches its fields?
pub fn refund_request_is_signed(body: &Value, secret: &str) -> bool {
    let given = text(body, "vnp_SecureHash");
    !given.is_empty()
        && given.eq_ignore_ascii_case(&hmac_sha512(
            secret,
            &signing_data(body, &REQUEST_SIGNED_FIELDS),
        ))
}

/// A refund response with `vnp_SecureHash` filled in over the documented fields.
pub fn signed_refund_response(secret: &str, mut body: Value) -> Value {
    let hash = hmac_sha512(secret, &signing_data(&body, &RESPONSE_SIGNED_FIELDS));
    body["vnp_SecureHash"] = json!(hash);
    body
}

/// What VNPay answers when it refunded `request`.
pub fn refund_success(secret: &str, request: &Value) -> Value {
    signed_refund_response(
        secret,
        json!({
            "vnp_ResponseId": "resp-1",
            "vnp_Command": "refund",
            "vnp_ResponseCode": "00",
            "vnp_Message": "Refund Success",
            "vnp_TmnCode": text(request, "vnp_TmnCode"),
            "vnp_TxnRef": text(request, "vnp_TxnRef"),
            "vnp_Amount": request["vnp_Amount"],
            "vnp_BankCode": "NCB",
            "vnp_PayDate": "20261008101500",
            "vnp_TransactionNo": "14000099",
            "vnp_TransactionType": "02",
            "vnp_TransactionStatus": "00",
            "vnp_OrderInfo": text(request, "vnp_OrderInfo"),
        }),
    )
}

/// How the stub answers a refund request.
#[derive(Debug, Clone)]
pub enum Behaviour {
    /// refunds it, if the request is signed correctly
    Refunds,
    /// answers with an error code (91 transaction not found, 94 duplicate request, ...)
    Refuses { code: &'static str, message: &'static str },
    /// accepted but not finished, `vnp_TransactionStatus` 05
    StillProcessing,
    /// HTTP 500
    Down,
    /// refunds, but only after this long
    Slow(Duration),
    /// claims success with a signature made with the wrong secret
    ForgedSuccess,
    /// 200 with a body that is not JSON
    Garbled,
}

struct Responder {
    secret: String,
    behaviour: Behaviour,
}

impl Respond for Responder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap_or(Value::Null);
        // like VNPay: a request we can't verify is refused before anything else
        if !refund_request_is_signed(&body, &self.secret) {
            return ResponseTemplate::new(200).set_body_json(json!({
                "vnp_ResponseCode": "97",
                "vnp_Message": "Invalid signature",
            }));
        }
        match &self.behaviour {
            Behaviour::Refunds => {
                ResponseTemplate::new(200).set_body_json(refund_success(&self.secret, &body))
            }
            Behaviour::Refuses { code, message } => ResponseTemplate::new(200)
                .set_body_json(json!({ "vnp_ResponseCode": code, "vnp_Message": message })),
            Behaviour::StillProcessing => {
                let mut reply = refund_success(&self.secret, &body);
                reply["vnp_TransactionStatus"] = json!("05");
                ResponseTemplate::new(200)
                    .set_body_json(signed_refund_response(&self.secret, reply))
            }
            Behaviour::Down => ResponseTemplate::new(500).set_body_string("upstream broke"),
            Behaviour::Slow(delay) => ResponseTemplate::new(200)
                .set_body_json(refund_success(&self.secret, &body))
                .set_delay(*delay),
            Behaviour::ForgedSuccess => ResponseTemplate::new(200)
                .set_body_json(refund_success("not-the-merchants-secret", &body)),
            Behaviour::Garbled => ResponseTemplate::new(200).set_body_string("<html>oops</html>"),
        }
    }
}

/// A running fake of VNPay's merchant API. Starts out refusing nothing and refunding everything.
pub struct VnPayStub {
    server: MockServer,
}

impl VnPayStub {
    pub async fn start() -> Self {
        let stub = VnPayStub {
            server: MockServer::start().await,
        };
        stub.behave(Behaviour::Refunds).await;
        stub
    }

    /// Where our service should send refunds: set it as `VNPAY_API_URL`.
    pub fn api_url(&self) -> String {
        format!("{}{API_PATH}", self.server.uri())
    }

    /// Switches how the stub answers from now on. Also forgets the requests received so far.
    pub async fn behave(&self, behaviour: Behaviour) {
        self.server.reset().await;
        Mock::given(method("POST"))
            .and(path(API_PATH))
            .respond_with(Responder {
                secret: HASH_SECRET.to_string(),
                behaviour,
            })
            .mount(&self.server)
            .await;
    }

    /// Every refund request received so far, as the JSON body we sent.
    pub async fn refund_requests(&self) -> Vec<Value> {
        self.server
            .received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .map(|r| serde_json::from_slice(&r.body).unwrap_or(Value::Null))
            .collect()
    }
}
