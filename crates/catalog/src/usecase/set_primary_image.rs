use crate::domain::cache::CatalogCache;
use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct SetPrimaryImageUsecase {
    repo: Arc<dyn CatalogRepository>,
    cache: Arc<dyn CatalogCache>,
}

impl SetPrimaryImageUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>, cache: Arc<dyn CatalogCache>) -> Self {
        Self { repo, cache }
    }

    pub async fn execute(&self, product_id: Uuid, image_id: Uuid) -> Result<(), AppError> {
        self.repo.set_primary_image(product_id, image_id).await?;

        // Invalidate cache
        if let Ok(Some(product_with_variants)) = self.repo.get_by_id(product_id).await {
            let _ = self.cache.delete_product(&product_with_variants.product.slug).await;
        }

        Ok(())
    }
}
