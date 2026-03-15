use crate::error::AppError;
use async_trait::async_trait;
use futures::future::BoxFuture;

/// The Bridge: Every DB transaction or connection must implement this.
pub trait DbExecutor: Send + Sync {}

/// The Orchestrator: This trait is dyn-compatible.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn run_atomic(
        &self,
        f: Box<
            dyn for<'a> FnOnce(&'a mut dyn DbExecutor) -> BoxFuture<'a, Result<(), AppError>>
                + Send,
        >,
    ) -> Result<(), AppError>;

    async fn run_read_only(
        &self,
        f: Box<
            dyn for<'a> FnOnce(&'a mut dyn DbExecutor) -> BoxFuture<'a, Result<(), AppError>>
                + Send,
        >,
    ) -> Result<(), AppError>;
}
