//! Component tests with the marketing routes running inside the test process: real HTTP on a
//! free port, real use cases, an in-memory repository instead of Postgres.
//! They check who may manage coupons and prices, and that validating a coupon stays public.

use async_trait::async_trait;
use marketing::domain::model::Coupon;
use marketing::domain::repository::CouponRepository;
use marketing::routes::{init, init_public, MarketingUsecase};
use marketing::usecase::{
    apply_category_discount::ApplyCategoryDiscountUsecase, create_coupon::CreateCouponUsecase,
    deactivate_coupon::DeactivateCouponUsecase, delete_coupon::DeleteCouponUsecase,
    list_coupons::ListCouponsUsecase, validate_coupon::ValidateCouponUsecase,
};
use serde_json::{json, Value};
use shared::database::DbExecutor;
use shared::error::AppError;
use shared::testing::{stand_in_for_auth, NoopUnitOfWork};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Default)]
struct FakeCoupons {
    coupons: Mutex<Vec<Coupon>>,
    discounts: Mutex<Vec<(Uuid, i32)>>,
}

#[async_trait]
impl CouponRepository for FakeCoupons {
    async fn create_coupon(&self, _: &mut dyn DbExecutor, coupon: &Coupon) -> Result<(), AppError> {
        self.coupons.lock().unwrap().push(coupon.clone());
        Ok(())
    }
    async fn get_coupon_by_code(&self, _: &mut dyn DbExecutor, code: &str) -> Result<Coupon, AppError> {
        self.coupons
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.code == code)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Coupon not found".into()))
    }
    async fn update_coupon(&self, _: &mut dyn DbExecutor, coupon: &Coupon) -> Result<(), AppError> {
        let mut all = self.coupons.lock().unwrap();
        if let Some(slot) = all.iter_mut().find(|c| c.id == coupon.id) {
            *slot = coupon.clone();
        }
        Ok(())
    }
    async fn list_coupons(&self, _: &mut dyn DbExecutor) -> Result<Vec<Coupon>, AppError> {
        Ok(self.coupons.lock().unwrap().clone())
    }
    async fn delete_coupon(&self, _: &mut dyn DbExecutor, id: Uuid) -> Result<(), AppError> {
        self.coupons.lock().unwrap().retain(|c| c.id != id);
        Ok(())
    }
    async fn apply_category_discount(&self, _: &mut dyn DbExecutor, category_id: Uuid, percent: i32) -> Result<(), AppError> {
        self.discounts.lock().unwrap().push((category_id, percent));
        Ok(())
    }
}

struct TestApp {
    base: String,
    repo: Arc<FakeCoupons>,
}

struct Reply {
    status: u16,
    body: Value,
}

/// Mounted like the gateway does: coupon validation public, the rest behind auth.
async fn spawn_app() -> TestApp {
    let repo = Arc::new(FakeCoupons::default());
    let uow = Arc::new(NoopUnitOfWork);
    let state = MarketingUsecase::new(
        Arc::new(CreateCouponUsecase::new(repo.clone(), uow.clone())),
        Arc::new(ValidateCouponUsecase::new(repo.clone(), uow.clone())),
        Arc::new(ListCouponsUsecase::new(repo.clone(), uow.clone())),
        Arc::new(DeactivateCouponUsecase::new(repo.clone(), uow.clone())),
        Arc::new(DeleteCouponUsecase::new(repo.clone(), uow)),
        Arc::new(ApplyCategoryDiscountUsecase::new(repo.clone())),
    );
    let app = axum::Router::new()
        .nest("/api/v1/marketing", init_public().with_state(state.clone()))
        .nest(
            "/api/v1/marketing",
            init()
                .with_state(state)
                .layer(axum::middleware::from_fn(stand_in_for_auth)),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/api/v1/marketing", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestApp { base, repo }
}

impl TestApp {
    /// `role` is `None` for a call with no login
    async fn call(&self, method: reqwest::Method, path: &str, role: Option<&str>, body: Option<Value>) -> Reply {
        let mut req = reqwest::Client::new().request(method, format!("{}{path}", self.base));
        if let Some(role) = role {
            req = req.header("x-test-role", role);
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        let res = req.send().await.unwrap();
        Reply { status: res.status().as_u16(), body: res.json().await.unwrap_or(Value::Null) }
    }

    fn coupon(&self, code: &str) {
        self.repo.coupons.lock().unwrap().push(Coupon::new(code.into(), 5_000, 10).unwrap());
    }
}

fn coupon_body(code: &str) -> Value {
    json!({ "code": code, "discount_amount": 5_000, "max_uses": 10 })
}

fn discount_body() -> Value {
    json!({ "category_id": Uuid::new_v4(), "percent": 90 })
}

#[tokio::test]
async fn anyone_can_validate_a_coupon_without_logging_in() {
    let app = spawn_app().await;
    app.coupon("SUMMER");

    let r = app.call(reqwest::Method::POST, "/coupons/validate", None, Some(json!({ "code": "SUMMER" }))).await;

    assert_eq!(r.status, 200);
    assert_eq!((r.body["is_valid"].clone(), r.body["discount_amount"].clone()), (json!(true), json!(5_000)));
}

#[tokio::test]
async fn managing_coupons_and_prices_answers_401_without_a_login() {
    let app = spawn_app().await;
    app.coupon("SUMMER");

    for (method, path, body) in [
        (reqwest::Method::GET, "/coupons", None),
        (reqwest::Method::POST, "/coupons", Some(coupon_body("FREE"))),
        (reqwest::Method::PATCH, "/coupons/SUMMER/deactivate", Some(json!({}))),
        (reqwest::Method::DELETE, "/coupons/SUMMER", None),
        (reqwest::Method::POST, "/discounts/category", Some(discount_body())),
    ] {
        let r = app.call(method.clone(), path, None, body).await;
        assert_eq!(r.status, 401, "{method} {path}");
    }
    assert_eq!(app.repo.coupons.lock().unwrap().len(), 1, "nothing was created or removed");
    assert!(app.repo.discounts.lock().unwrap().is_empty());
}

#[tokio::test]
async fn a_customer_cannot_manage_coupons_or_change_prices() {
    let app = spawn_app().await;
    app.coupon("SUMMER");

    for (method, path, body) in [
        (reqwest::Method::GET, "/coupons", None),
        (reqwest::Method::POST, "/coupons", Some(coupon_body("FREE"))),
        (reqwest::Method::PATCH, "/coupons/SUMMER/deactivate", Some(json!({}))),
        (reqwest::Method::DELETE, "/coupons/SUMMER", None),
        (reqwest::Method::POST, "/discounts/category", Some(discount_body())),
    ] {
        let r = app.call(method.clone(), path, Some("customer"), body).await;
        assert_eq!(r.status, 403, "{method} {path}");
    }
    let coupons = app.repo.coupons.lock().unwrap();
    assert_eq!(coupons.len(), 1, "no coupon was created or deleted");
    assert!(coupons[0].is_active, "and the one there is still active");
    assert!(app.repo.discounts.lock().unwrap().is_empty(), "no price was changed");
}

#[tokio::test]
async fn staff_manage_coupons_but_only_admins_change_prices() {
    let app = spawn_app().await;

    let created = app.call(reqwest::Method::POST, "/coupons", Some("staff"), Some(coupon_body("SPRING"))).await;
    let listed = app.call(reqwest::Method::GET, "/coupons", Some("staff"), None).await;
    let deactivated = app.call(reqwest::Method::PATCH, "/coupons/SPRING/deactivate", Some("staff"), Some(json!({}))).await;
    let deleted = app.call(reqwest::Method::DELETE, "/coupons/SPRING", Some("staff"), None).await;
    let staff_discount = app.call(reqwest::Method::POST, "/discounts/category", Some("staff"), Some(discount_body())).await;

    assert_eq!((created.status, listed.status, deactivated.status, deleted.status), (200, 200, 200, 200));
    assert_eq!(listed.body.as_array().unwrap().len(), 1);
    assert_eq!(staff_discount.status, 403);
    assert!(app.repo.discounts.lock().unwrap().is_empty());

    let admin_discount = app.call(reqwest::Method::POST, "/discounts/category", Some("admin"), Some(discount_body())).await;
    assert_eq!(admin_discount.status, 200);
    assert_eq!(app.repo.discounts.lock().unwrap().len(), 1);
}
