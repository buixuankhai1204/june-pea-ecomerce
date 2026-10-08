//! Postgres and Redis for the tests that need the real thing. Each starts a container with
//! Testcontainers (needs Docker) unless the matching env var points at one that is already
//! running, which is what CI usually does. Use a throwaway database for `TEST_DATABASE_URL`,
//! the service migrates it on startup.
//!
//! The container stops when the returned value is dropped.

use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::{Redis, REDIS_PORT};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};

pub struct TestPostgres {
    pub url: String,
    _container: Option<ContainerAsync<Postgres>>,
}

pub async fn start_postgres() -> TestPostgres {
    if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        return TestPostgres {
            url,
            _container: None,
        };
    }
    let node = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("failed to start Postgres container (is Docker running?)");
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    TestPostgres {
        url: format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres"),
        _container: Some(node),
    }
}

pub struct TestRedis {
    pub url: String,
    _container: Option<ContainerAsync<Redis>>,
}

pub async fn start_redis() -> TestRedis {
    if let Ok(url) = std::env::var("TEST_REDIS_URL") {
        return TestRedis {
            url,
            _container: None,
        };
    }
    let node = Redis::default()
        .with_tag("7-alpine")
        .start()
        .await
        .expect("failed to start Redis container (is Docker running?)");
    let port = node.get_host_port_ipv4(REDIS_PORT).await.unwrap();
    TestRedis {
        url: format!("redis://127.0.0.1:{port}"),
        _container: Some(node),
    }
}
