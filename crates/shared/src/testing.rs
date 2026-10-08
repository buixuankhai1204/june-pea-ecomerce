//! Test doubles for code that takes a `UnitOfWork`, so use cases can run without a database.
//!
//! Only pair these with in-memory repositories. A Postgres repository casts the executor it is
//! given to a transaction (see `SqlxExecutor::from_executor`), and `NoopExecutor` is not one.

use crate::database::{DbExecutor, UnitOfWork};
use crate::error::AppError;
use async_trait::async_trait;
use futures::future::BoxFuture;

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
