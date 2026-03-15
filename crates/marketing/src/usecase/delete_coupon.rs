use crate::domain::repository::CouponRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;

pub struct DeleteCouponUsecase {
    repo: Arc<dyn CouponRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl DeleteCouponUsecase {
    pub fn new(repo: Arc<dyn CouponRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, code: &str) -> Result<(), AppError> {
        let repo = self.repo.clone();
        let code_str = code.to_string();
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    let coupon = repo.get_coupon_by_code(exec, &code_str).await?;
                    repo.delete_coupon(exec, coupon.id).await
                })
            }))
            .await
    }
}
