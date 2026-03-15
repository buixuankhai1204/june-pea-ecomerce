use crate::domain::repository::CouponRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;

pub struct DeactivateCouponUsecase {
    repo: Arc<dyn CouponRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl DeactivateCouponUsecase {
    pub fn new(repo: Arc<dyn CouponRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, code: &str) -> Result<(), AppError> {
        let repo = self.repo.clone();
        let code_clone = code.to_string();

        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let mut coupon = repo.get_coupon_by_code(exec, &code_clone).await?;

                    coupon.deactivate()?;

                    repo.update_coupon(exec, &coupon).await
                })
            }))
            .await?;

        Ok(())
    }
}
