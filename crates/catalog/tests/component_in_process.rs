//! Component tests with the catalog routes running inside the test process: real HTTP on a
//! free port, real use cases, a fake repository and cache instead of Postgres and Redis.
//! They check that browsing is open to everyone and that changing the catalog is for staff.

use async_trait::async_trait;
use catalog::domain::cache::CatalogCache;
use catalog::domain::catalog_repository::CatalogRepository;
use catalog::domain::model::{Category, ProductWithVariants};
use catalog::routes::{init, CatalogUsecase};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use shared::error::AppError;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Reads answer "nothing there"; every write is recorded by name so a test can see whether it
/// got through.
#[derive(Default)]
struct FakeCatalog {
    writes: Mutex<Vec<&'static str>>,
}

impl FakeCatalog {
    fn wrote(&self, what: &'static str) -> Result<(), AppError> {
        self.writes.lock().unwrap().push(what);
        Ok(())
    }
}

#[async_trait]
impl CatalogRepository for FakeCatalog {
    async fn get_by_slug(&self, _: &str) -> Result<Option<ProductWithVariants>, AppError> {
        Ok(None)
    }
    async fn get_by_id(&self, _: Uuid) -> Result<Option<ProductWithVariants>, AppError> {
        Ok(None)
    }
    async fn list_all(&self, _: i64, _: i64) -> Result<Vec<ProductWithVariants>, AppError> {
        Ok(vec![])
    }
    async fn count_all(&self) -> Result<i64, AppError> {
        Ok(0)
    }
    async fn create_category(&self, _: Uuid, _: &str, _: &str, _: Option<Uuid>) -> Result<(), AppError> {
        self.wrote("create_category")
    }
    async fn list_categories(&self) -> Result<Vec<Category>, AppError> {
        Ok(vec![])
    }
    async fn create_product(&self, _: Uuid, _: Uuid, _: &str, _: &str, _: Option<&str>) -> Result<(), AppError> {
        self.wrote("create_product")
    }
    async fn update_product(&self, _: Uuid, _: Uuid, _: &str, _: &str, _: Option<&str>) -> Result<(), AppError> {
        self.wrote("update_product")
    }
    async fn delete_product(&self, _: Uuid) -> Result<(), AppError> {
        self.wrote("delete_product")
    }
    async fn delete_category(&self, _: Uuid) -> Result<(), AppError> {
        self.wrote("delete_category")
    }
    async fn search_products(&self, _: &str, _: i64, _: i64) -> Result<Vec<ProductWithVariants>, AppError> {
        Ok(vec![])
    }
    async fn get_categories_by_parent(&self, _: Option<Uuid>) -> Result<Vec<Category>, AppError> {
        Ok(vec![])
    }
    async fn create_variant(&self, _: Uuid, _: Uuid, _: &str, _: &str, _: Decimal, _: Option<Decimal>, _: Value) -> Result<(), AppError> {
        self.wrote("create_variant")
    }
    async fn update_variant(&self, _: Uuid, _: &str, _: &str, _: Decimal, _: Option<Decimal>, _: Value) -> Result<(), AppError> {
        self.wrote("update_variant")
    }
    async fn delete_variant(&self, _: Uuid) -> Result<(), AppError> {
        self.wrote("delete_variant")
    }
}

struct NoCache;

#[async_trait]
impl CatalogCache for NoCache {
    async fn get_product(&self, _: &str) -> Result<Option<ProductWithVariants>, AppError> {
        Ok(None)
    }
    async fn set_product(&self, _: &str, _: &ProductWithVariants) -> Result<(), AppError> {
        Ok(())
    }
    async fn delete_product(&self, _: &str) -> Result<(), AppError> {
        Ok(())
    }
}

struct TestApp {
    base: String,
    repo: Arc<FakeCatalog>,
}

/// Mounted like the gateway mounts it: the routes are open, and the auth middleware only
/// *reads* a login if there is one (unlike `shared::testing::stand_in_for_auth`, which insists
/// on one). Here `x-test-role` stands in for the token, and a request without it goes through
/// as an anonymous visitor.
async fn spawn_app() -> TestApp {
    let repo = Arc::new(FakeCatalog::default());
    let state = CatalogUsecase::new(repo.clone(), Arc::new(NoCache));
    let app = axum::Router::new()
        .nest("/api/v1/catalog", init().with_state(state))
        .layer(axum::middleware::from_fn(
            |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
                if let Some(role) = req.headers().get("x-test-role").and_then(|v| v.to_str().ok()).map(str::to_owned) {
                    req.extensions_mut().insert(shared::auth::UserClaims { sub: Uuid::new_v4(), exp: usize::MAX, role });
                }
                next.run(req).await
            },
        ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/api/v1/catalog", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestApp { base, repo }
}

impl TestApp {
    async fn call(&self, method: reqwest::Method, path: &str, role: Option<&str>, body: Value) -> u16 {
        let mut req = reqwest::Client::new().request(method, format!("{}{path}", self.base)).json(&body);
        if let Some(role) = role {
            req = req.header("x-test-role", role);
        }
        req.send().await.unwrap().status().as_u16()
    }

    fn writes(&self) -> Vec<&'static str> {
        self.repo.writes.lock().unwrap().clone()
    }
}

fn category() -> Value {
    json!({ "name": "Hats" })
}

/// every route that changes the catalog, with a plausible body
fn writes() -> Vec<(reqwest::Method, String, Value)> {
    let id = Uuid::new_v4();
    let product = json!({ "category_id": Uuid::new_v4(), "name": "Cap", "slug": "cap", "description": null });
    let variant = json!({ "sku": "CAP-1", "name": "Cap", "base_price": "10.00", "sale_price": null, "attributes": {} });
    vec![
        (reqwest::Method::POST, "/categories".into(), category()),
        (reqwest::Method::DELETE, format!("/categories/{id}"), json!({})),
        (reqwest::Method::POST, "/products".into(), product.clone()),
        (reqwest::Method::PATCH, format!("/products/{id}"), product),
        (reqwest::Method::DELETE, format!("/products/{id}"), json!({})),
        (reqwest::Method::POST, "/variants".into(), json!({ "product_id": id, "sku": "CAP-1", "name": "Cap", "base_price": "10.00", "sale_price": null, "attributes": {} })),
        (reqwest::Method::PATCH, format!("/variants/{id}"), variant),
        (reqwest::Method::DELETE, format!("/variants/{id}"), json!({})),
        (reqwest::Method::POST, format!("/products/{id}/images/upload"), json!({})),
    ]
}

#[tokio::test]
async fn anyone_can_browse_without_logging_in() {
    let app = spawn_app().await;

    for path in ["/products", "/categories", "/categories/tree", "/products/search?q=cap"] {
        let status = app.call(reqwest::Method::GET, path, None, json!(null)).await;
        assert_eq!(status, 200, "GET {path}");
    }
    assert_eq!(app.call(reqwest::Method::GET, "/products/slug/nothing", None, json!(null)).await, 404);
}

#[tokio::test]
async fn changing_the_catalog_answers_401_to_a_visitor_who_is_not_logged_in() {
    let app = spawn_app().await;

    for (method, path, body) in writes() {
        assert_eq!(app.call(method.clone(), &path, None, body).await, 401, "{method} {path}");
    }
    assert!(app.writes().is_empty(), "nothing was written");
}

#[tokio::test]
async fn changing_the_catalog_answers_403_to_a_customer() {
    let app = spawn_app().await;

    for (method, path, body) in writes() {
        assert_eq!(app.call(method.clone(), &path, Some("customer"), body).await, 403, "{method} {path}");
    }
    assert!(app.writes().is_empty(), "nothing was written");
}

#[tokio::test]
async fn staff_and_admins_can_change_the_catalog() {
    let app = spawn_app().await;
    let id = Uuid::new_v4();

    let created = app.call(reqwest::Method::POST, "/categories", Some("staff"), category()).await;
    let deleted = app.call(reqwest::Method::DELETE, &format!("/products/{id}"), Some("admin"), json!({})).await;

    assert_eq!((created, deleted), (200, 200));
    assert_eq!(app.writes(), vec!["create_category", "delete_product"]);
}
