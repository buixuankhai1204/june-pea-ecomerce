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
use shared::testing::{stand_in_for_auth, NoopUnitOfWork};
use std::sync::Arc;
use uuid::Uuid;

struct TestApp {
    base: String,
    /// logged in as an admin, so the tests that aren't about permissions don't think about them
    http: reqwest::Client,
    repo: Arc<InMemoryOrderRepository>,
}

/// A client that sends the headers `stand_in_for_auth` turns into a login with that role.
fn client_as(role: &str, user: Uuid) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-test-role", role.parse().unwrap());
    headers.insert("x-test-user", user.to_string().parse().unwrap());
    reqwest::Client::builder().default_headers(headers).build().unwrap()
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
    // like the gateway: the routes sit behind the auth middleware
    let app = axum::Router::new()
        .nest("/api/v1/ordering", init().with_state(state))
        .layer(axum::middleware::from_fn(stand_in_for_auth));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/api/v1/ordering", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestApp { base, http: client_as("admin", Uuid::new_v4()), repo }
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

// ── who may do what ──────────────────────────────────────────────────────────────────────

impl TestApp {
    async fn as_user(&self, http: &reqwest::Client, method: reqwest::Method, path: &str, body: Option<Value>) -> Reply {
        let mut req = http.request(method, format!("{}{path}", self.base));
        if let Some(body) = body {
            req = req.json(&body);
        }
        self.send(req).await
    }

    async fn status_of(&self, id: Uuid) -> Value {
        self.get(&format!("/orders/{id}")).await.body["status"].clone()
    }
}

#[tokio::test]
async fn every_ordering_route_answers_401_without_a_login() {
    let app = spawn_app().await;
    let nobody = reqwest::Client::new();
    let id = Uuid::new_v4();
    let item = json!({ "customer_id": null, "items": shirt(1, 1_000) });

    for (method, path, body) in [
        (reqwest::Method::POST, "/orders".to_string(), Some(item)),
        (reqwest::Method::GET, "/orders".to_string(), None),
        (reqwest::Method::GET, format!("/orders/{id}"), None),
        (reqwest::Method::DELETE, format!("/orders/{id}"), None),
        (reqwest::Method::PATCH, format!("/orders/{id}/status"), Some(json!({ "status": "Completed" }))),
        (reqwest::Method::PATCH, format!("/orders/{id}/note"), Some(json!({ "note": "x" }))),
        (reqwest::Method::GET, format!("/orders/customer/{id}"), None),
        (reqwest::Method::GET, format!("/orders/customer/{id}/recent"), None),
    ] {
        let r = app.as_user(&nobody, method.clone(), &path, body).await;
        assert_eq!(r.status, 401, "{method} {path}");
    }
    assert_eq!(app.get("/orders").await.body, json!([]), "nothing was stored");
}

#[tokio::test]
async fn a_customer_always_orders_for_themself() {
    let app = spawn_app().await;
    let (me, someone_else) = (Uuid::new_v4(), Uuid::new_v4());
    let customer = client_as("customer", me);

    // no customer_id: the order is theirs, not a guest's
    let own = app
        .as_user(&customer, reqwest::Method::POST, "/orders", Some(json!({ "items": shirt(1, 1_000) })))
        .await;
    assert_eq!(own.status, 200, "{}", own.body);
    let own_id: Uuid = own.body["order_id"].as_str().unwrap().parse().unwrap();
    assert_eq!(app.get(&format!("/orders/{own_id}")).await.body["customer_id"], me.to_string());

    // someone else's id: refused, and nothing is stored
    let forged = app
        .as_user(&customer, reqwest::Method::POST, "/orders", Some(json!({ "customer_id": someone_else, "items": shirt(1, 1_000) })))
        .await;
    assert_eq!(forged.status, 403);
    assert_eq!(app.get("/orders").await.body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn staff_can_place_an_order_for_any_customer_or_for_a_guest() {
    let app = spawn_app().await;
    let customer = Uuid::new_v4();
    let staff = client_as("staff", Uuid::new_v4());

    let for_customer = app
        .as_user(&staff, reqwest::Method::POST, "/orders", Some(json!({ "customer_id": customer, "items": shirt(1, 1_000) })))
        .await;
    let for_guest = app.as_user(&staff, reqwest::Method::POST, "/orders", Some(json!({ "items": shirt(1, 1_000) }))).await;

    assert_eq!((for_customer.status, for_guest.status), (200, 200));
    let id: Uuid = for_customer.body["order_id"].as_str().unwrap().parse().unwrap();
    let guest: Uuid = for_guest.body["order_id"].as_str().unwrap().parse().unwrap();
    assert_eq!(app.get(&format!("/orders/{id}")).await.body["customer_id"], customer.to_string());
    assert_eq!(app.get(&format!("/orders/{guest}")).await.body["customer_id"], Value::Null);
}

#[tokio::test]
async fn a_customer_can_read_and_cancel_their_own_order_but_not_anyone_elses() {
    let app = spawn_app().await;
    let (me, other) = (Uuid::new_v4(), Uuid::new_v4());
    let (mine, theirs, guests) = (app.place_ok(Some(me)).await, app.place_ok(Some(other)).await, app.place_ok(None).await);
    let customer = client_as("customer", me);

    assert_eq!(app.as_user(&customer, reqwest::Method::GET, &format!("/orders/{mine}"), None).await.status, 200);
    for foreign in [theirs, guests] {
        let read = app.as_user(&customer, reqwest::Method::GET, &format!("/orders/{foreign}"), None).await;
        let cancel = app.as_user(&customer, reqwest::Method::DELETE, &format!("/orders/{foreign}"), None).await;
        assert_eq!((read.status, cancel.status), (403, 403), "order {foreign}");
        assert_eq!(app.status_of(foreign).await, "Pending", "the cancel did not go through");
    }

    let cancel_own = app.as_user(&customer, reqwest::Method::DELETE, &format!("/orders/{mine}"), None).await;
    assert_eq!(cancel_own.status, 200);
    assert_eq!(app.status_of(mine).await, "Cancelled");
}

#[tokio::test]
async fn a_customer_cannot_list_another_customers_orders() {
    let app = spawn_app().await;
    let (me, other) = (Uuid::new_v4(), Uuid::new_v4());
    app.place_ok(Some(me)).await;
    app.place_ok(Some(other)).await;
    let customer = client_as("customer", me);

    let own = app.as_user(&customer, reqwest::Method::GET, &format!("/orders/customer/{me}"), None).await;
    let own_recent = app.as_user(&customer, reqwest::Method::GET, &format!("/orders/customer/{me}/recent"), None).await;
    let foreign = app.as_user(&customer, reqwest::Method::GET, &format!("/orders/customer/{other}"), None).await;
    let foreign_recent = app.as_user(&customer, reqwest::Method::GET, &format!("/orders/customer/{other}/recent"), None).await;

    assert_eq!((own.status, own_recent.status), (200, 200));
    assert_eq!(own.body.as_array().unwrap().len(), 1);
    assert_eq!((foreign.status, foreign_recent.status), (403, 403));
}

#[tokio::test]
async fn a_customer_cannot_list_all_orders_or_change_status_or_notes() {
    let app = spawn_app().await;
    let me = Uuid::new_v4();
    let id = app.place_ok(Some(me)).await;
    let customer = client_as("customer", me);

    let all = app.as_user(&customer, reqwest::Method::GET, "/orders", None).await;
    let status = app
        .as_user(&customer, reqwest::Method::PATCH, &format!("/orders/{id}/status"), Some(json!({ "status": "Completed" })))
        .await;
    let note = app
        .as_user(&customer, reqwest::Method::PATCH, &format!("/orders/{id}/note"), Some(json!({ "note": "mine" })))
        .await;

    assert_eq!((all.status, status.status, note.status), (403, 403, 403));
    assert_eq!(app.status_of(id).await, "Pending", "an order can't be marked paid by its buyer");
    assert_eq!(app.get(&format!("/orders/{id}")).await.body["note"], Value::Null);
}

#[tokio::test]
async fn staff_can_read_cancel_and_manage_any_order() {
    let app = spawn_app().await;
    let id = app.place_ok(Some(Uuid::new_v4())).await;
    let staff = client_as("staff", Uuid::new_v4());

    let read = app.as_user(&staff, reqwest::Method::GET, &format!("/orders/{id}"), None).await;
    let all = app.as_user(&staff, reqwest::Method::GET, "/orders", None).await;
    let note = app
        .as_user(&staff, reqwest::Method::PATCH, &format!("/orders/{id}/note"), Some(json!({ "note": "packed" })))
        .await;
    let cancel = app.as_user(&staff, reqwest::Method::DELETE, &format!("/orders/{id}"), None).await;

    assert_eq!((read.status, all.status, note.status, cancel.status), (200, 200, 200, 200));
    assert_eq!(app.status_of(id).await, "Cancelled");
}
