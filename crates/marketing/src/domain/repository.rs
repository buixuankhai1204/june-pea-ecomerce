use crate::domain::model::Coupon;
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError};

#[async_trait]
pub trait CouponRepository: Send + Sync {
    async fn create_coupon(
        &self,
        exec: &mut dyn DbExecutor,
        coupon: &Coupon,
    ) -> Result<(), AppError>;
    async fn get_coupon_by_code(
        &self,
        exec: &mut dyn DbExecutor,
        code: &str,
    ) -> Result<Coupon, AppError>;
    async fn update_coupon(
        &self,
        exec: &mut dyn DbExecutor,
        coupon: &Coupon,
    ) -> Result<(), AppError>;
    async fn list_coupons(&self, exec: &mut dyn DbExecutor) -> Result<Vec<Coupon>, AppError>;
    async fn delete_coupon(
        &self,
        exec: &mut dyn DbExecutor,
        id: uuid::Uuid,
    ) -> Result<(), AppError>;
}
