//! Test doubles for code that takes a `UnitOfWork`, so use cases can run without a database.
//!
//! Only pair these with in-memory repositories. A Postgres repository casts the executor it is
//! given to a transaction (see `SqlxExecutor::from_executor`), and `NoopExecutor` is not one.

use crate::auth::UserClaims;
use crate::database::{DbExecutor, UnitOfWork};
use crate::error::AppError;
use async_trait::async_trait;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use futures::future::BoxFuture;
use uuid::Uuid;

/// Stands in for the gateway's JWT middleware in component tests that mount a crate's routes
/// on their own. Mount it with `axum::middleware::from_fn`.
///
/// The caller comes from headers instead of a token: `x-test-role` is the role (no header
/// means nobody is logged in) and `x-test-user` is the user id, a fresh one if left out.
pub async fn stand_in_for_auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // read the headers in their own scope: a borrow of the request must not live across the
    // `.await` below, or the future stops being `Send` and axum refuses it
    let (role, sub) = {
        let header = |name: &str| req.headers().get(name).and_then(|v| v.to_str().ok()).map(str::to_owned);
        let role = header("x-test-role").ok_or(StatusCode::UNAUTHORIZED)?;
        let sub = header("x-test-user").and_then(|v| v.parse().ok()).unwrap_or_else(Uuid::new_v4);
        (role, sub)
    };
    req.extensions_mut().insert(UserClaims { sub, exp: usize::MAX, role });
    Ok(next.run(req).await)
}

pub struct NoopExecutor;

impl DbExecutor for NoopExecutor {}

/// Runs the closure straight away: no transaction, nothing to commit or roll back.
#[derive(Default, Clone, Copy)]
pub struct NoopUnitOfWork;

#[async_trait]
impl UnitOfWork for NoopUnitOfWork {
    async fn run_atomic(
        &self,
        f: Box<
            dyn for<'a> FnOnce(&'a mut dyn DbExecutor) -> BoxFuture<'a, Result<(), AppError>>
                + Send,
        >,
    ) -> Result<(), AppError> {
        f(&mut NoopExecutor).await
    }

    async fn run_read_only(
        &self,
        f: Box<
            dyn for<'a> FnOnce(&'a mut dyn DbExecutor) -> BoxFuture<'a, Result<(), AppError>>
                + Send,
        >,
    ) -> Result<(), AppError> {
        f(&mut NoopExecutor).await
    }
}
