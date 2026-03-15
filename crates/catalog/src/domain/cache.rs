use crate::domain::model::ProductWithVariants;
use async_trait::async_trait;
use shared::AppError;

#[async_trait]
pub trait CatalogCache: Send + Sync {
    async fn get_product(&self, slug: &str) -> Result<Option<ProductWithVariants>, AppError>;
    async fn set_product(&self, slug: &str, data: &ProductWithVariants) -> Result<(), AppError>;
    async fn delete_product(&self, slug: &str) -> Result<(), AppError>;
}
