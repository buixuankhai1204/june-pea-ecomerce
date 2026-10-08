//! Component tests with the ordering routes running inside the test process: real HTTP on a
//! free port, real use cases and domain rules, an in-memory repository instead of Postgres.
//! Covers routing, JSON, status codes and the error mapping. `integration_test.rs` covers SQL.

use ordering::infrastructure::persistence::memory::InMemoryOrderRepository;
use ordering::routes::{init, OrderingUsecase};
use ordering::usecase::{
    cancel_order::CancelOrderUsecase, get_order::GetOrderUsecase,
    list_all_orders::ListAllOrdersUsecase,
    list_customer_recent_orders::ListCustomerRecentOrdersUsecase, list_orders::ListOrdersUsecase,
    place_order::PlaceOrderUsecase, update_order_note::UpdateOrderNoteUsecase,
    update_order_status::UpdateOrderStatusUsecase,
};
use serde_json::{json, Value};
use shared::testing::NoopUnitOfWork;
use std::sync::Arc;
use uuid::Uuid;

struct TestApp {
    base: String,
    http: reqwest::Client,
    repo: Arc<InMemoryOrderRepository>,
}

async fn spawn_app() -> TestApp {
    let repo = Arc::new(InMemoryOrderRepository::default());
    let uow = Arc::new(NoopUnitOfWork);
    let state = OrderingUsecase::new(
        Arc::new(PlaceOrderUsecase::new(repo.clone(), uow.clone())),
        Arc::new(CancelOrderUsecase::new(repo.clone(), uow.clone())),
        Arc::new(GetOrderUsecase::new(repo.clone(), uow.clone())),
        Arc::new(ListOrdersUsecase::new(repo.clone(), uow.clone())),
        Arc::new(UpdateOrderStatusUsecase::new(repo.clone(), uow.clone())),
        Arc::new(ListAllOrdersUsecase::new(repo.clone(), uow.clone())),
        Arc::new(UpdateOrderNoteUsecase::new(repo.clone(), uow.clone())),
        Arc::new(ListCustomerRecentOrdersUsecase::new(repo.clone(), uow)),
    );
    let app = axum::Router::new().nest("/api/v1/ordering", init().with_state(state));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/api/v1/ordering", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestApp { base, http: reqwest::Client::new(), repo }
}

struct Reply {
    status: u16,
    body: Value,
}

impl TestApp {
    async fn send(&self, req: reqwest::RequestBuilder) -> Reply {
        let res = req.send().await.unwrap();
        Reply { status: res.status().as_u16(), body: res.json().await.unwrap_or(Value::Null) }
    }

    async fn place(&self, customer: Option<Uuid>, items: Value) -> Reply {
        let req = self.http.post(format!("{}/orders", self.base));
        self.send(req.json(&json!({ "customer_id": customer, "items": items }))).await
    }

    async fn get(&self, path: &str) -> Reply {
        self.send(self.http.get(format!("{}{path}", self.base))).await
    }

    async fn place_ok(&self, customer: Option<Uuid>) -> Uuid {
        let r = self.place(customer, shirt(2, 50_000)).await;
        assert_eq!(r.status, 200, "{}", r.body);
        r.body["order_id"].as_str().unwrap().parse().unwrap()
    }
}

fn shirt(quantity: i32, unit_price: i64) -> Value {
    json!([{ "variant_id": Uuid::new_v4(), "quantity": quantity, "unit_price": unit_price }])
}

#[tokio::test]
async fn placing_an_order_returns_its_id_and_stores_a_pending_order_with_its_items() {
    let app = spawn_app().await;

    let id = app.place_ok(None).await;

    let order = app.get(&format!("/orders/{id}")).await;
    assert_eq!(order.status, 200);
    assert_eq!(order.body["status"], "Pending");
    assert_eq!(order.body["total"], 100_000);
    assert_eq!(app.repo.items_of(id).len(), 1);
}

#[tokio::test]
async fn an_order_without_items_is_400() {
    let app = spawn_app().await;

    assert_eq!(app.place(None, json!([])).await.status, 400);
}

#[tokio::test]
async fn an_item_with_zero_quantity_is_400_and_nothing_is_stored() {
    let app = spawn_app().await;

    assert_eq!(app.place(None, shirt(0, 50_000)).await.status, 400);
    assert_eq!(app.get("/orders").await.body, json!([]));
}

#[tokio::test]
async fn an_unknown_order_is_404() {
    let app = spawn_app().await;

    assert_eq!(app.get(&format!("/orders/{}", Uuid::new_v4())).await.status, 404);
}

#[tokio::test]
async fn a_pending_order_can_be_cancelled_once() {
    let app = spawn_app().await;
    let id = app.place_ok(None).await;
    let delete = || app.send(app.http.delete(format!("{}/orders/{id}", app.base)));

    let first = delete().await;
    let second = delete().await;

    assert_eq!((first.status, first.body), (200, json!(true)));
    assert_eq!(app.get(&format!("/orders/{id}")).await.body["status"], "Cancelled");
    assert_eq!(second.status, 400, "already cancelled");
}

#[tokio::test]
async fn cancelling_an_unknown_order_is_404() {
    let app = spawn_app().await;

    let r = app.send(app.http.delete(format!("{}/orders/{}", app.base, Uuid::new_v4()))).await;

    assert_eq!(r.status, 404);
}

#[tokio::test]
async fn the_status_can_be_changed_and_an_unknown_status_is_rejected() {
    let app = spawn_app().await;
    let id = app.place_ok(None).await;
    let patch = |status: &str| {
        app.send(app.http.patch(format!("{}/orders/{id}/status", app.base)).json(&json!({ "status": status })))
    };

    assert_eq!(patch("Completed").await.status, 200);
    assert_eq!(app.get(&format!("/orders/{id}")).await.body["status"], "Completed");
    assert!(patch("Shipped").await.status >= 400);
}

#[tokio::test]
async fn a_note_can_be_added_to_an_order() {
    let app = spawn_app().await;
    let id = app.place_ok(None).await;

    let r = app
        .send(app.http.patch(format!("{}/orders/{id}/note", app.base)).json(&json!({ "note": "gift wrap" })))
        .await;

    assert_eq!(r.status, 200);
    assert_eq!(app.get(&format!("/orders/{id}")).await.body["note"], "gift wrap");
}

#[tokio::test]
async fn a_customer_sees_only_their_own_orders() {
    let app = spawn_app().await;
    let (me, someone_else) = (Uuid::new_v4(), Uuid::new_v4());
    app.place_ok(Some(me)).await;
    app.place_ok(Some(me)).await;
    app.place_ok(Some(someone_else)).await;

    let mine = app.get(&format!("/orders/customer/{me}")).await;

    assert_eq!(mine.body.as_array().unwrap().len(), 2);
    assert!(mine.body.as_array().unwrap().iter().all(|o| o["customer_id"] == me.to_string()));
    assert_eq!(app.get("/orders").await.body.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn recent_orders_are_newest_first_and_limited() {
    let app = spawn_app().await;
    let me = Uuid::new_v4();
    let mut ids = vec![];
    for _ in 0..3 {
        ids.push(app.place_ok(Some(me)).await);
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    let recent = app.get(&format!("/orders/customer/{me}/recent?limit=2")).await;

    let got: Vec<_> = recent.body.as_array().unwrap().iter().map(|o| o["id"].as_str().unwrap().to_string()).collect();
    assert_eq!(got, vec![ids[2].to_string(), ids[1].to_string()]);
}
