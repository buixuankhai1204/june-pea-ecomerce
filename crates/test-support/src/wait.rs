//! Waiting for things that happen in another process.

use std::future::Future;
use std::net::TcpListener;
use std::time::{Duration, Instant};

/// Asks the OS for a free port, so tests can start services side by side.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Polls `check` until it returns Some, or panics after `timeout`.
pub async fn eventually<T, F, Fut>(timeout: Duration, mut check: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(value) = check().await {
            return value;
        }
        assert!(Instant::now() < deadline, "timed out waiting for condition");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
