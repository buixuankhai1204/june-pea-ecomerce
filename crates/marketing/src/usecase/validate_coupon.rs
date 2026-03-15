use crate::domain::{model::Coupon, repository::CouponRepository};
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;

pub struct ValidateCouponUsecase {
    repo: Arc<dyn CouponRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl ValidateCouponUsecase {
    pub fn new(repo: Arc<dyn CouponRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, code: &str) -> Result<Coupon, AppError> {
        let repo = self.repo.clone();
        let code_clone = code.to_string();

        let fetched_coupon = Arc::new(tokio::sync::Mutex::new(None));
        let fetched_coupon_clone = fetched_coupon.clone();

        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let coupon = repo.get_coupon_by_code(exec, &code_clone).await?;
                    *fetched_coupon_clone.lock().await = Some(coupon);
                    Ok(())
                })
            }))
            .await?;

        let coupon = fetched_coupon.lock().await.take().unwrap();

        if !coupon.is_valid() {
            return Err(AppError::Validation(
                "Coupon is no longer valid or has reached maximum uses".to_string(),
            ));
        }

        Ok(coupon)
    }
}
