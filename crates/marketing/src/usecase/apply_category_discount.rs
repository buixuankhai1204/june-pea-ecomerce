use crate::domain::repository::CouponRepository;
use shared::{database::DbExecutor, error::AppError};
use uuid::Uuid;
use std::sync::Arc;

pub struct ApplyCategoryDiscountUsecase {
    repo: Arc<dyn CouponRepository>,
}

impl ApplyCategoryDiscountUsecase {
    pub fn new(repo: Arc<dyn CouponRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, category_id: Uuid, percent: i32) -> Result<(), AppError> {
        // Here we'd normally use a UnitOfWork if we had one for Marketing that supports multiple executors,
        // but PostgresCouponRepository uses self.pool directly in the implementation I just wrote for simplicity
        // as the existing pattern in this crate matches this.
        
        // Actually, let's stick to the repo method I defined.
        // The _exec parameter is unused in my new impl but that matches some other methods there.
        
        struct DummyExecutor;
        impl DbExecutor for DummyExecutor {}
        let mut exec = DummyExecutor;

        self.repo.apply_category_discount(&mut exec, category_id, percent).await
    }
}
