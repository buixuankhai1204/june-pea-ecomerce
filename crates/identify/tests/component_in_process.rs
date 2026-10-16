//! Component tests with the identity routes running inside the test process: real HTTP on a
//! free port, real use cases, an in-memory user repository instead of Postgres.
//! They check who may call what, and that no response carries a password hash.

use async_trait::async_trait;
use identify::domain::model::{User, UserMembership};
use identify::domain::user_repository::UserRepository;
use identify::routes::{init, init_public, IdentityState};
use identify::usecase::{
    auth::AuthUsecase, change_password::ChangePasswordUsecase, create_staff::CreateStaffUsecase,
    delete_user::DeleteUserUsecase, get_me::GetMeUsecase,
    get_membership_summary::GetMembershipSummaryUsecase, list_memberships::ListMembershipsUsecase,
    list_staff::ListStaffUsecase, list_users::ListUsersUsecase,
    update_profile::UpdateProfileUsecase,
};
use serde_json::{json, Value};
use shared::error::AppError;
use shared::testing::stand_in_for_auth;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const HASH: &str = "$argon2id$v=19$leaky-hash-that-must-never-be-served";

#[derive(Default)]
struct FakeUsers(Mutex<Vec<User>>);

#[async_trait]
impl UserRepository for FakeUsers {
    async fn create_user(&self, user: &User) -> Result<(), AppError> {
        self.0.lock().unwrap().push(User {
            id: user.id,
            email: user.email.clone(),
            password_hash: user.password_hash.clone(),
            role: user.role.clone(),
        });
        Ok(())
    }
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        Ok(self.0.lock().unwrap().iter().find(|u| u.email == email).map(copy))
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        Ok(self.0.lock().unwrap().iter().find(|u| u.id == id).map(copy))
    }
    async fn update_user(&self, _: &User) -> Result<(), AppError> {
        Ok(())
    }
    async fn list_users(&self) -> Result<Vec<User>, AppError> {
        Ok(self.0.lock().unwrap().iter().map(copy).collect())
    }
    async fn list_staff(&self) -> Result<Vec<User>, AppError> {
        Ok(self.0.lock().unwrap().iter().filter(|u| u.role != "customer").map(copy).collect())
    }
    async fn list_memberships(&self) -> Result<Vec<UserMembership>, AppError> {
        Ok(vec![])
    }
    async fn delete_user(&self, id: Uuid) -> Result<(), AppError> {
        self.0.lock().unwrap().retain(|u| u.id != id);
        Ok(())
    }
}

fn copy(u: &User) -> User {
    User { id: u.id, email: u.email.clone(), password_hash: u.password_hash.clone(), role: u.role.clone() }
}

/// What the gateway's `AppState` does for identity: build each use case over the repository.
#[derive(Clone)]
struct TestState {
    repo: Arc<dyn UserRepository>,
}

impl IdentityState for TestState {
    fn auth_service(&self) -> Arc<AuthUsecase> {
        Arc::new(AuthUsecase::new(self.repo.clone()))
    }
    fn get_me_usecase(&self) -> Arc<GetMeUsecase> {
        Arc::new(GetMeUsecase::new(self.repo.clone()))
    }
    fn update_profile_usecase(&self) -> Arc<UpdateProfileUsecase> {
        Arc::new(UpdateProfileUsecase::new(self.repo.clone()))
    }
    fn list_users_usecase(&self) -> Arc<ListUsersUsecase> {
        Arc::new(ListUsersUsecase::new(self.repo.clone()))
    }
    fn list_memberships_usecase(&self) -> Arc<ListMembershipsUsecase> {
        Arc::new(ListMembershipsUsecase::new(self.repo.clone()))
    }
    fn get_membership_summary_usecase(&self) -> Arc<GetMembershipSummaryUsecase> {
        Arc::new(GetMembershipSummaryUsecase::new(self.repo.clone()))
    }
    fn change_password_usecase(&self) -> Arc<ChangePasswordUsecase> {
        Arc::new(ChangePasswordUsecase::new(self.repo.clone()))
    }
    fn delete_user_usecase(&self) -> Arc<DeleteUserUsecase> {
        Arc::new(DeleteUserUsecase::new(self.repo.clone()))
    }
    fn list_staff_usecase(&self) -> Arc<ListStaffUsecase> {
        Arc::new(ListStaffUsecase::new(self.repo.clone()))
    }
    fn create_staff_usecase(&self) -> Arc<CreateStaffUsecase> {
        Arc::new(CreateStaffUsecase::new(self.repo.clone()))
    }
}

struct TestApp {
    base: String,
    http: reqwest::Client,
    repo: Arc<FakeUsers>,
}

struct Reply {
    status: u16,
    body: Value,
    raw: String,
}

/// Mounted like the gateway does: register/login public, the rest behind auth.
async fn spawn_app() -> TestApp {
    let repo = Arc::new(FakeUsers::default());
    let state = TestState { repo: repo.clone() };
    let app = axum::Router::new()
        .nest("/api/v1/auth", init_public::<TestState>())
        .nest(
            "/api/v1/identity",
            init::<TestState>().layer(axum::middleware::from_fn(stand_in_for_auth)),
        )
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}/api/v1", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    TestApp { base, http: reqwest::Client::new(), repo }
}

impl TestApp {
    fn add(&self, email: &str, role: &str) -> Uuid {
        let mut user = User::new(email.into(), HASH.into());
        user.role = role.into();
        let id = user.id;
        self.repo.0.lock().unwrap().push(user);
        id
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Reply {
        let res = req.send().await.unwrap();
        let status = res.status().as_u16();
        let raw = res.text().await.unwrap();
        Reply { status, body: serde_json::from_str(&raw).unwrap_or(Value::Null), raw }
    }

    /// `who` is `None` for a call with no login
    async fn call(&self, method: reqwest::Method, path: &str, who: Option<(&str, Uuid)>, body: Option<Value>) -> Reply {
        let mut req = self.http.request(method, format!("{}{path}", self.base));
        if let Some((role, id)) = who {
            req = req.header("x-test-role", role).header("x-test-user", id.to_string());
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        self.send(req).await
    }

    async fn get(&self, path: &str, who: Option<(&str, Uuid)>) -> Reply {
        self.call(reqwest::Method::GET, path, who, None).await
    }
}

#[tokio::test]
async fn registering_needs_no_login() {
    let app = spawn_app().await;

    let r = app
        .call(
            reqwest::Method::POST,
            "/auth/register",
            None,
            Some(json!({ "email": "new@example.test", "password": "pw-123456", "password_confirm": "pw-123456" })),
        )
        .await;

    assert_eq!(r.status, 200, "{}", r.raw);
    assert_eq!(app.repo.0.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn the_public_mount_does_not_expose_the_user_admin_routes() {
    let app = spawn_app().await;
    app.add("c@example.test", "customer");

    // these used to be reachable without a token, because the whole identity router was
    // mounted under /auth
    for path in ["/auth/users", "/auth/staff", "/auth/memberships", "/auth/me"] {
        let r = app.get(path, None).await;
        assert_eq!(r.status, 404, "GET {path}: {}", r.raw);
    }
    let r = app.call(reqwest::Method::DELETE, &format!("/auth/users/{}", Uuid::new_v4()), None, None).await;
    assert_eq!(r.status, 404);
    let r = app
        .call(reqwest::Method::POST, "/auth/staff", None, Some(json!({ "email": "x@example.test", "password": "pw" })))
        .await;
    assert_eq!(r.status, 404);
    assert_eq!(app.repo.0.lock().unwrap().len(), 1, "nothing was created or removed");
}

#[tokio::test]
async fn every_protected_identity_route_answers_401_without_a_login() {
    let app = spawn_app().await;
    let id = Uuid::new_v4();

    for (method, path) in [
        (reqwest::Method::GET, "/identity/me".to_string()),
        (reqwest::Method::GET, "/identity/users".to_string()),
        (reqwest::Method::DELETE, format!("/identity/users/{id}")),
        (reqwest::Method::GET, "/identity/memberships".to_string()),
        (reqwest::Method::GET, "/identity/memberships/summary".to_string()),
        (reqwest::Method::GET, "/identity/staff".to_string()),
    ] {
        let r = app.call(method.clone(), &path, None, None).await;
        assert_eq!(r.status, 401, "{method} {path}");
    }
}

#[tokio::test]
async fn a_customer_cannot_list_users_manage_staff_or_see_memberships() {
    let app = spawn_app().await;
    let me = app.add("c@example.test", "customer");
    let other = app.add("other@example.test", "customer");
    let who = Some(("customer", me));

    assert_eq!(app.get("/identity/users", who).await.status, 403);
    assert_eq!(app.get("/identity/staff", who).await.status, 403);
    assert_eq!(app.get("/identity/memberships", who).await.status, 403);
    assert_eq!(app.get("/identity/memberships/summary", who).await.status, 403);
    let create = app
        .call(reqwest::Method::POST, "/identity/staff", who, Some(json!({ "email": "evil@example.test", "password": "pw" })))
        .await;
    assert_eq!(create.status, 403);
    let delete = app.call(reqwest::Method::DELETE, &format!("/identity/users/{other}"), who, None).await;
    assert_eq!(delete.status, 403);

    let users = app.repo.0.lock().unwrap();
    assert_eq!(users.len(), 2, "no staff was created and nobody was deleted");
    assert!(users.iter().all(|u| u.role == "customer"));
}

#[tokio::test]
async fn staff_can_see_memberships_but_not_the_user_list() {
    let app = spawn_app().await;
    let staff = app.add("s@example.test", "staff");
    let who = Some(("staff", staff));

    assert_eq!(app.get("/identity/memberships", who).await.status, 200);
    assert_eq!(app.get("/identity/users", who).await.status, 403);
    assert_eq!(app.get("/identity/staff", who).await.status, 403);
}

#[tokio::test]
async fn the_user_list_is_for_admins_and_never_contains_a_password_hash() {
    let app = spawn_app().await;
    let admin = app.add("a@example.test", "admin");
    app.add("c@example.test", "customer");

    let r = app.get("/identity/users", Some(("admin", admin))).await;

    assert_eq!(r.status, 200, "{}", r.raw);
    assert_eq!(r.body.as_array().unwrap().len(), 2);
    assert!(!r.raw.contains(HASH), "{}", r.raw);
    assert!(!r.raw.contains("password"), "{}", r.raw);
    assert_eq!(r.body[0]["email"], "a@example.test");
    assert_eq!(r.body[0]["role"], "admin");
}

#[tokio::test]
async fn me_returns_the_callers_own_user_without_the_password_hash() {
    let app = spawn_app().await;
    let me = app.add("c@example.test", "customer");

    let r = app.get("/identity/me", Some(("customer", me))).await;

    assert_eq!(r.status, 200, "{}", r.raw);
    assert_eq!((r.body["id"].as_str().unwrap(), r.body["email"].as_str().unwrap()), (me.to_string().as_str(), "c@example.test"));
    assert!(!r.raw.contains(HASH), "{}", r.raw);
    assert!(!r.raw.contains("password"), "{}", r.raw);
}

#[tokio::test]
async fn an_admin_can_create_a_staff_account_and_delete_a_user() {
    let app = spawn_app().await;
    let admin = app.add("a@example.test", "admin");
    let victim = app.add("c@example.test", "customer");
    let who = Some(("admin", admin));

    let created = app
        .call(reqwest::Method::POST, "/identity/staff", who, Some(json!({ "email": "s@example.test", "password": "pw-123456" })))
        .await;
    let deleted = app.call(reqwest::Method::DELETE, &format!("/identity/users/{victim}"), who, None).await;

    assert_eq!(created.status, 200, "{}", created.raw);
    assert!(!created.raw.contains("argon2"), "{}", created.raw);
    assert_eq!(deleted.status, 200, "{}", deleted.raw);
    let roles: Vec<_> = app.repo.0.lock().unwrap().iter().map(|u| (u.email.clone(), u.role.clone())).collect();
    assert_eq!(roles, vec![("a@example.test".into(), "admin".into()), ("s@example.test".into(), "staff".into())]);
}
