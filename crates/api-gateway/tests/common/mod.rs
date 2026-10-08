//! Helpers for the tests that run the compiled `api-gateway` binary: Postgres and Redis from
//! `test_support::containers`, a fake VNPay, and the process itself.

#![allow(dead_code)]

use serde_json::{json, Value};
use sqlx::PgPool;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use test_support::containers::{start_postgres, start_redis, TestPostgres, TestRedis};
use test_support::vnpay::{VnPayStub, HASH_SECRET, TMN_CODE};
use test_support::wait::free_port;
use uuid::Uuid;

/// A variant from `migrations/05_seed.sql`: "Pro Laptop 16 inch", 140_000 on sale.
pub const SEEDED_VARIANT: &str = "cccccccc-cccc-cccc-cccc-cccccccccccc";
pub const SEEDED_VARIANT_PRICE: i64 = 140_000;

/// The running binary with everything it talks to. Killed (and the containers stopped) on drop.
pub struct Gateway {
    child: Child,
    pub base: String,
    pub http: reqwest::Client,
    pub pool: PgPool,
    pub vnpay: VnPayStub,
    _pg: TestPostgres,
    _redis: TestRedis,
}

impl Gateway {
    pub async fn start() -> Self {
        let pg = start_postgres().await;
        let redis = start_redis().await;
        let vnpay = VnPayStub::start().await;
        let port = free_port();

        // an empty working directory, so dotenv doesn't find the repo's .env and fill in
        // anything we didn't set
        let workdir = std::env::temp_dir().join(format!("api-gateway-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&workdir).unwrap();

        let child = Command::new(env!("CARGO_BIN_EXE_api-gateway"))
            .current_dir(&workdir)
            .env_clear()
            .env("PORT", port.to_string())
            .env("DATABASE_URL", &pg.url)
            .env("REDIS_URL", &redis.url)
            .env("JWT_SECRET", "component-test-jwt-secret")
            .env("VNPAY_TMNCODE", TMN_CODE)
            .env("VNPAY_HASH_SECRET", HASH_SECRET)
            .env("VNPAY_PAYMENT_URL", "https://pay.example/vpcpay.html")
            .env("VNPAY_RETURN_URL", "https://shop.example/payment/return")
            .env("VNPAY_IPN_URL", "https://shop.example/api/v1/payment/vnpay/ipn")
            .env("VNPAY_API_URL", vnpay.api_url())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to start api-gateway");

        let mut gateway = Gateway {
            child,
            base: format!("http://127.0.0.1:{port}"),
            http: reqwest::Client::new(),
            pool: PgPool::connect(&pg.url).await.unwrap(),
            vnpay,
            _pg: pg,
            _redis: redis,
        };
        gateway.wait_until_serving().await;
        gateway
    }

    /// The gateway has no health route; the public catalog list answers once it is up and
    /// migrated, so poll that instead of sleeping.
    async fn wait_until_serving(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                panic!("api-gateway exited early with {status}");
            }
            let url = format!("{}/api/v1/catalog/products", self.base);
            if let Ok(res) = self.http.get(url).send().await {
                if res.status().is_success() {
                    return;
                }
            }
            assert!(Instant::now() < deadline, "api-gateway did not come up in time");
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}/api/v1{path}", self.base)
    }

    /// Registers through the API and logs in. `role` is applied straight in the database, the
    /// API has no way to make an admin. Returns the user id and the JWT.
    pub async fn user(&self, role: &str) -> (Uuid, String) {
        let email = format!("{}@example.test", Uuid::new_v4());
        let res = self
            .http
            .post(self.url("/auth/register"))
            .json(&json!({ "email": email, "password": "pw-123456", "password_confirm": "pw-123456" }))
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success(), "register: {}", res.status());

        sqlx::query("UPDATE identify.users SET role = $1 WHERE email = $2")
            .bind(role)
            .bind(&email)
            .execute(&self.pool)
            .await
            .unwrap();
        let id: Uuid = sqlx::query_scalar("SELECT id FROM identify.users WHERE email = $1")
            .bind(&email)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        let login: Value = self
            .http
            .post(self.url("/auth/login"))
            .json(&json!({ "email": email, "password": "pw-123456" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        (id, login["token"].as_str().expect("login returned a token").to_string())
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
