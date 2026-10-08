//! Helpers for the payment test suites: config, in-memory stand-ins for the order side, and the
//! app on a real socket.

#![allow(dead_code)]

use async_trait::async_trait;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{from_fn, Next};
use axum::response::Response;
use axum::Router;
use payment::config::PaymentConfig;
use payment::domain::{
    GatewayError, OrderLookup, OrderState, OrderSummary, PaymentGateway, RefundReceipt,
    RefundRequest,
};
use payment::infrastructure::persistence::memory::InMemoryPaymentRepository;
use payment::infrastructure::vnpay::{VnPayClient, VnPayGateway};
use payment::routes::PaymentUsecase;
use payment::usecase::{
    create_vnpay_qr::CreateVnPayQrUsecase, get_payment_status::GetPaymentStatusUsecase,
    handle_vnpay_ipn::HandleVnPayIpnUsecase, refund_payment::RefundPaymentUsecase,
};
use serde_json::Value;
use shared::auth::UserClaims;
use shared::error::AppError;
use shared::testing::NoopUnitOfWork;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use test_support::vnpay::{paid_ipn_query, VnPayStub, HASH_SECRET, TMN_CODE};
use uuid::Uuid;

pub fn test_config(api_url: String) -> PaymentConfig {
    PaymentConfig {
        tmn_code: TMN_CODE.into(),
        hash_secret: HASH_SECRET.into(),
        payment_url: "https://pay.example/vpcpay.html".into(),
        return_url: "https://shop.example/payment/return".into(),
        ipn_url: "https://shop.example/api/v1/payment/vnpay/ipn".into(),
        default_locale: "vn".into(),
        order_type: "other".into(),
        qr_expiry_minutes: 15,
        api_url,
        server_ip: "127.0.0.1".into(),
        api_timeout_secs: 5,
    }
}

/// The ordering service as payment sees it (`OrderLookup`), in memory.
#[derive(Default)]
pub struct FakeOrders {
    orders: Mutex<HashMap<Uuid, OrderSummary>>,
}

impl FakeOrders {
    pub fn add(&self, total: i64) -> Uuid {
        let id = Uuid::new_v4();
        self.orders.lock().unwrap().insert(
            id,
            OrderSummary {
                id,
                total,
                state: OrderState::Pending,
            },
        );
        id
    }

    pub fn state_of(&self, id: Uuid) -> OrderState {
        self.orders.lock().unwrap()[&id].state
    }

    pub fn set_state(&self, id: Uuid, state: OrderState) {
        self.orders.lock().unwrap().get_mut(&id).unwrap().state = state;
    }
}

#[async_trait]
impl OrderLookup for FakeOrders {
    async fn find(&self, order_id: Uuid) -> Result<OrderSummary, AppError> {
        self.orders
            .lock()
            .unwrap()
            .get(&order_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Order {order_id} not found")))
    }

    async fn mark_completed(&self, order_id: Uuid) -> Result<(), AppError> {
        self.set_state(order_id, OrderState::Completed);
        Ok(())
    }
}

/// Stands in for the gateway's JWT middleware: the role comes from an `x-test-role` header
/// instead of a token, and no header means not logged in.
async fn stand_in_for_auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let role = req
        .headers()
        .get("x-test-role")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();
    req.extensions_mut().insert(UserClaims {
        sub: ADMIN_ID,
        exp: usize::MAX,
        role,
    });
    Ok(next.run(req).await)
}

/// whoever `stand_in_for_auth` says is calling
pub const ADMIN_ID: Uuid = Uuid::from_u128(0xad);

/// The payment routes mounted like the gateway does (IPN public, the rest behind auth), on a
/// free port, with in-memory repositories and a fake VNPay behind the real HTTP gateway.
pub struct TestApp {
    pub base: String,
    pub http: reqwest::Client,
    pub vnpay: VnPayStub,
    pub payments: Arc<InMemoryPaymentRepository>,
    pub orders: Arc<FakeOrders>,
}

pub async fn spawn_app() -> TestApp {
    let vnpay = VnPayStub::start().await;
    let config = test_config(vnpay.api_url());
    let payments = Arc::new(InMemoryPaymentRepository::default());
    let orders = Arc::new(FakeOrders::default());
    let uow = Arc::new(NoopUnitOfWork);

    let client = Arc::new(VnPayClient::new(config.clone()));
    let state = PaymentUsecase::new(
        Arc::new(CreateVnPayQrUsecase::new(
            payments.clone(),
            uow.clone(),
            orders.clone(),
            client.clone(),
            config.clone(),
        )),
        Arc::new(GetPaymentStatusUsecase::new(payments.clone(), uow.clone())),
        Arc::new(HandleVnPayIpnUsecase::new(
            payments.clone(),
            uow.clone(),
            client,
            orders.clone(),
        )),
        Arc::new(RefundPaymentUsecase::new(
            payments.clone(),
            uow.clone(),
            Arc::new(VnPayGateway::new(&config).unwrap()),
        )),
    );

    let public = Router::new().nest(
        "/api/v1/payment",
        payment::routes::init_ipn().with_state(state.clone()),
    );
    let protected = Router::new()
        .nest("/api/v1/payment", payment::routes::init().with_state(state))
        .layer(from_fn(stand_in_for_auth));
    let app = Router::new().merge(public).merge(protected);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    TestApp {
        base,
        http: reqwest::Client::new(),
        vnpay,
        payments,
        orders,
    }
}

pub struct Reply {
    pub status: u16,
    pub body: Value,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}/api/v1/payment{path}", self.base)
    }

    async fn reply(res: reqwest::Response) -> Reply {
        let status = res.status().as_u16();
        Reply {
            status,
            body: res.json().await.unwrap_or(Value::Null),
        }
    }

    /// a pending order worth `total_vnd`
    pub async fn seed_order(&self, total_vnd: i64) -> Uuid {
        self.orders.add(total_vnd)
    }

    pub async fn request_qr(&self, order_id: Uuid, role: &str) -> Reply {
        let res = self
            .http
            .post(self.url("/vnpay/qr"))
            .header("x-test-role", role)
            .json(&serde_json::json!({ "order_id": order_id }))
            .send()
            .await
            .unwrap();
        Self::reply(res).await
    }

    pub async fn payment_status(&self, order_id: Uuid, role: &str) -> Reply {
        let res = self
            .http
            .get(self.url(&format!("/orders/{order_id}")))
            .header("x-test-role", role)
            .send()
            .await
            .unwrap();
        Self::reply(res).await
    }

    /// VNPay calling our IPN endpoint with `query` (no auth, VNPay has no token)
    pub async fn ipn(&self, query: &str) -> Reply {
        let res = self
            .http
            .get(format!("{}?{query}", self.url("/vnpay/ipn")))
            .send()
            .await
            .unwrap();
        Self::reply(res).await
    }

    pub async fn refund(&self, order_id: Uuid, role: &str) -> Reply {
        let res = self
            .http
            .post(self.url(&format!("/orders/{order_id}/refund")))
            .header("x-test-role", role)
            .send()
            .await
            .unwrap();
        Self::reply(res).await
    }

    /// An order that went through checkout and was paid: QR requested, then VNPay's IPN.
    /// Returns the order id and the `vnp_TxnRef` VNPay knows the payment by.
    pub async fn paid_order(&self, total_vnd: i64) -> (Uuid, String) {
        let order_id = self.seed_order(total_vnd).await;
        let qr = self.request_qr(order_id, "customer").await;
        assert_eq!(qr.status, 200, "{}", qr.body);
        let txn_ref = qr.body["txn_ref"].as_str().unwrap().to_string();

        let ipn = self
            .ipn(&paid_ipn_query(HASH_SECRET, &txn_ref, total_vnd))
            .await;
        assert_eq!(ipn.body["rsp_code"], "00", "{}", ipn.body);
        (order_id, txn_ref)
    }
}

/// A provider that refunds anything, for tests where the refund call is not the point.
pub struct AlwaysRefunds;

#[async_trait]
impl PaymentGateway for AlwaysRefunds {
    async fn refund(&self, _request: RefundRequest) -> Result<RefundReceipt, GatewayError> {
        Ok(RefundReceipt {
            transaction_no: "14000099".into(),
            message: "Refund Success".into(),
        })
    }
}
