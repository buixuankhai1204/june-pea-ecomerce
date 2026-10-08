//! Component tests with the inventory routes running inside the test process: real HTTP on a
//! free port, real use cases, an in-memory repository instead of Postgres.
//! `integration_test.rs` covers the SQL.

use inventory::infrastructure::persistence::memory::InMemoryInventoryRepository;
use inventory::routes::{init, InventoryUsecase};
use serde_json::{json, Value};
use shared::testing::NoopUnitOfWork;
use std::sync::Arc;
use uuid::Uuid;

struct TestApp {
    base: String,
    http: reqwest::Client,
    repo: Arc<InMemoryInventoryRepository>,
}

struct Reply {
    status: u16,
    body: Value,
}

async fn spawn_app() -> TestApp {
    let repo = Arc::new(InMemoryInventoryRepository::default());
    let state = InventoryUsecase::new(repo.clone(), Arc::new(NoopUnitOfWork));
    let app = axum::Router::new().nest("/api/v1/inventory", init().with_state(state));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/api/v1/inventory", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestApp { base, http: reqwest::Client::new(), repo }
}

impl TestApp {
    async fn post(&self, path: &str, body: Value) -> Reply {
        self.reply(self.http.post(format!("{}{path}", self.base)).json(&body)).await
    }

    async fn get(&self, path: &str) -> Reply {
        self.reply(self.http.get(format!("{}{path}", self.base))).await
    }

    async fn reply(&self, req: reqwest::RequestBuilder) -> Reply {
        let res = req.send().await.unwrap();
        Reply { status: res.status().as_u16(), body: res.json().await.unwrap_or(Value::Null) }
    }

    fn stocked(&self, quantity: i32) -> Uuid {
        let id = Uuid::new_v4();
        self.repo.set(id, quantity);
        id
    }
}

#[tokio::test]
async fn stock_of_a_variant_is_readable() {
    let app = spawn_app().await;
    let v = app.stocked(12);

    let r = app.get(&format!("/stock/{v}")).await;

    assert_eq!((r.status, r.body), (200, json!(12)));
}

#[tokio::test]
async fn decreasing_stock_changes_the_quantity() {
    let app = spawn_app().await;
    let v = app.stocked(10);

    let r = app.post("/decrease-stock", json!({ "variant_id": v, "amount": 4 })).await;

    assert_eq!((r.status, r.body), (200, json!(true)));
    assert_eq!(app.get(&format!("/stock/{v}")).await.body, json!(6));
}

#[tokio::test]
async fn decreasing_more_than_is_in_stock_is_409_and_changes_nothing() {
    let app = spawn_app().await;
    let v = app.stocked(2);

    let r = app.post("/decrease-stock", json!({ "variant_id": v, "amount": 3 })).await;

    assert_eq!(r.status, 409);
    assert_eq!(app.repo.quantity(v), Some(2));
}

#[tokio::test]
async fn amounts_must_be_positive() {
    let app = spawn_app().await;
    let v = app.stocked(5);

    for path in ["/decrease-stock", "/increase-stock"] {
        for amount in [0, -1] {
            let r = app.post(path, json!({ "variant_id": v, "amount": amount })).await;
            assert_eq!(r.status, 400, "{path} {amount}");
        }
    }
    assert_eq!(app.repo.quantity(v), Some(5));
}

#[tokio::test]
async fn increasing_stock_adds_to_the_quantity() {
    let app = spawn_app().await;
    let v = app.stocked(5);

    let r = app.post("/increase-stock", json!({ "variant_id": v, "amount": 20 })).await;

    assert_eq!(r.status, 200);
    assert_eq!(app.repo.quantity(v), Some(25));
}

#[tokio::test]
async fn stock_can_be_set_to_an_exact_quantity() {
    let app = spawn_app().await;
    let v = app.stocked(5);

    app.post("/update-stock", json!({ "variant_id": v, "quantity": 42 })).await;

    assert_eq!(app.repo.quantity(v), Some(42));
}

#[tokio::test]
async fn low_stock_alerts_use_the_threshold_and_default_to_ten() {
    let app = spawn_app().await;
    let (low, fine) = (app.stocked(3), app.stocked(50));

    let default = app.get("/stock/low-alerts").await;
    let wide = app.get("/stock/low-alerts?threshold=100").await;

    let ids = |r: &Reply| -> Vec<String> {
        r.body.as_array().unwrap().iter().map(|s| s["variant_id"].as_str().unwrap().to_string()).collect()
    };
    assert_eq!(ids(&default), vec![low.to_string()]);
    assert_eq!(ids(&wide).len(), 2);
    assert!(ids(&wide).contains(&fine.to_string()));
}

#[tokio::test]
async fn suppliers_can_be_created_listed_and_deleted() {
    let app = spawn_app().await;

    let created = app
        .post("/suppliers", json!({ "name": "Acme", "contact": "a@acme.test", "location": "Hanoi" }))
        .await;
    let listed = app.get("/suppliers").await;
    let id = listed.body[0]["id"].as_str().unwrap().to_string();
    let deleted = app.reply(app.http.delete(format!("{}/suppliers/{id}", app.base))).await;

    assert_eq!(created.status, 200);
    assert_eq!(listed.body[0]["name"], "Acme");
    assert_eq!(deleted.status, 200);
    assert_eq!(app.get("/suppliers").await.body, json!([]));
}
