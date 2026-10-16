//! Component tests with the inventory routes running inside the test process: real HTTP on a
//! free port, real use cases, an in-memory repository instead of Postgres.
//! `integration_test.rs` covers the SQL.

use inventory::infrastructure::persistence::memory::InMemoryInventoryRepository;
use inventory::routes::{init, InventoryUsecase};
use serde_json::{json, Value};
use shared::testing::{stand_in_for_auth, NoopUnitOfWork};
use std::sync::Arc;
use uuid::Uuid;

struct TestApp {
    base: String,
    /// logged in as staff, so the tests that aren't about permissions don't think about them
    http: reqwest::Client,
    repo: Arc<InMemoryInventoryRepository>,
}

/// A client that sends the headers `stand_in_for_auth` turns into a login with that role.
fn client_as(role: &str) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-test-role", role.parse().unwrap());
    reqwest::Client::builder().default_headers(headers).build().unwrap()
}

struct Reply {
    status: u16,
    body: Value,
}

async fn spawn_app() -> TestApp {
    let repo = Arc::new(InMemoryInventoryRepository::default());
    let state = InventoryUsecase::new(repo.clone(), Arc::new(NoopUnitOfWork));
    // like the gateway: the routes sit behind the auth middleware
    let app = axum::Router::new()
        .nest("/api/v1/inventory", init().with_state(state))
        .layer(axum::middleware::from_fn(stand_in_for_auth));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/api/v1/inventory", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestApp { base, http: client_as("staff"), repo }
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

// ── who may do what ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn every_inventory_route_answers_401_without_a_login() {
    let app = spawn_app().await;
    let nobody = reqwest::Client::new();
    let v = app.stocked(5);
    let write = json!({ "variant_id": v, "amount": 1 });

    for (method, path, body) in [
        (reqwest::Method::GET, format!("/stock/{v}"), None),
        (reqwest::Method::POST, "/decrease-stock".to_string(), Some(write.clone())),
        (reqwest::Method::POST, "/increase-stock".to_string(), Some(write)),
        (reqwest::Method::POST, "/update-stock".to_string(), Some(json!({ "variant_id": v, "quantity": 99 }))),
        (reqwest::Method::GET, "/list-all".to_string(), None),
        (reqwest::Method::GET, "/suppliers".to_string(), None),
        (reqwest::Method::POST, "/suppliers".to_string(), Some(json!({ "name": "n", "contact": "c", "location": "l" }))),
        (reqwest::Method::DELETE, format!("/suppliers/{}", Uuid::new_v4()), None),
        (reqwest::Method::GET, "/stock/low-alerts".to_string(), None),
    ] {
        let mut req = nobody.request(method.clone(), format!("{}{path}", app.base));
        if let Some(body) = body {
            req = req.json(&body);
        }
        assert_eq!(app.reply(req).await.status, 401, "{method} {path}");
    }
    assert_eq!(app.repo.quantity(v), Some(5), "nothing changed");
}

#[tokio::test]
async fn a_customer_can_see_a_variants_stock_but_not_change_it_or_see_the_back_office() {
    let app = spawn_app().await;
    let v = app.stocked(5);
    let customer = client_as("customer");
    let call = |method: reqwest::Method, path: String, body: Option<Value>| {
        let mut req = customer.request(method, format!("{}{path}", app.base));
        if let Some(body) = body {
            req = req.json(&body);
        }
        app.reply(req)
    };

    assert_eq!(call(reqwest::Method::GET, format!("/stock/{v}"), None).await.status, 200);
    for (method, path, body) in [
        (reqwest::Method::POST, "/decrease-stock".to_string(), Some(json!({ "variant_id": v, "amount": 5 }))),
        (reqwest::Method::POST, "/increase-stock".to_string(), Some(json!({ "variant_id": v, "amount": 5 }))),
        (reqwest::Method::POST, "/update-stock".to_string(), Some(json!({ "variant_id": v, "quantity": 999 }))),
        (reqwest::Method::GET, "/list-all".to_string(), None),
        (reqwest::Method::GET, "/suppliers".to_string(), None),
        (reqwest::Method::POST, "/suppliers".to_string(), Some(json!({ "name": "n", "contact": "c", "location": "l" }))),
        (reqwest::Method::DELETE, format!("/suppliers/{}", Uuid::new_v4()), None),
        (reqwest::Method::GET, "/stock/low-alerts".to_string(), None),
    ] {
        assert_eq!(call(method.clone(), path.clone(), body).await.status, 403, "{method} {path}");
    }
    assert_eq!(app.repo.quantity(v), Some(5), "the customer changed nothing");
}

#[tokio::test]
async fn admins_can_do_what_staff_can() {
    let app = spawn_app().await;
    let v = app.stocked(5);
    let admin = client_as("admin");

    let req = admin.post(format!("{}/decrease-stock", app.base)).json(&json!({ "variant_id": v, "amount": 2 }));

    assert_eq!(app.reply(req).await.status, 200);
    assert_eq!(app.repo.quantity(v), Some(3));
}
