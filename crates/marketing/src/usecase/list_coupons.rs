use crate::domain::{model::Coupon, repository::CouponRepository};
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;

pub struct ListCouponsUsecase {
    repo: Arc<dyn CouponRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl ListCouponsUsecase {
    pub fn new(repo: Arc<dyn CouponRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self) -> Result<Vec<Coupon>, AppError> {
        let repo = self.repo.clone();

        let fetched_coupons = Arc::new(tokio::sync::Mutex::new(None));
        let fetched_coupons_clone = fetched_coupons.clone();

        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let coupons = repo.list_coupons(exec).await?;
                    *fetched_coupons_clone.lock().await = Some(coupons);
                    Ok(())
                })
            }))
            .await?;

        let coupons = fetched_coupons.lock().await.take().unwrap();
        Ok(coupons)
    }
}
