use crate::database::{DbExecutor, UnitOfWork};
use crate::error::AppError;
use async_trait::async_trait;
use futures::future::BoxFuture;
use sqlx::PgPool;

/// The Concrete Wrapper for SQLx Transactions
pub struct SqlxExecutor<'a> {
    pub tx: sqlx::Transaction<'a, sqlx::Postgres>,
}

// Implement the trait without the Any conflict
impl<'a> DbExecutor for SqlxExecutor<'a> {}

impl<'a> SqlxExecutor<'a> {
    /// Internal helper to recover the executor from a trait object
    /// This is safe because we only use it inside the Postgres Infrastructure
    pub fn from_executor(exec: &mut dyn DbExecutor) -> &mut Self {
        unsafe { &mut *(exec as *mut dyn DbExecutor as *mut Self) }
    }
}

#[derive(Clone)]
pub struct PostgresUnitOfWork {
    pool: PgPool,
}

impl PostgresUnitOfWork {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWork for PostgresUnitOfWork {
    async fn run_atomic(
        &self,
        f: Box<
            dyn for<'a> FnOnce(&'a mut dyn DbExecutor) -> BoxFuture<'a, Result<(), AppError>>
                + Send,
        >,
    ) -> Result<(), AppError> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let mut executor = SqlxExecutor { tx };

        let result = f(&mut executor).await;

        match result {
            Ok(_) => {
                executor
                    .tx
                    .commit()
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                Ok(())
            }
            Err(e) => {
                let _ = executor.tx.rollback().await;
                Err(e)
            }
        }
    }

    async fn run_read_only(
        &self,
        f: Box<
            dyn for<'a> FnOnce(&'a mut dyn DbExecutor) -> BoxFuture<'a, Result<(), AppError>>
                + Send,
        >,
    ) -> Result<(), AppError> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let mut executor = SqlxExecutor { tx };

        let result = f(&mut executor).await;

        // For read-only, we always rollback (or commit, it doesn't matter much as long as no writes were made)
        let _ = executor.tx.rollback().await;

        result
    }
}
