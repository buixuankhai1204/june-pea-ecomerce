use crate::domain::cache::CatalogCache;
use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteProductImageUsecase {
    repo: Arc<dyn CatalogRepository>,
    cache: Arc<dyn CatalogCache>,
}

impl DeleteProductImageUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>, cache: Arc<dyn CatalogCache>) -> Self {
        Self { repo, cache }
    }

    pub async fn execute(&self, image_id: Uuid) -> Result<(), AppError> {
        // Find product_id before deleting
        let product_id = self.repo.get_product_id_by_image_id(image_id).await?;
        
        self.repo.delete_product_image(image_id).await?;

        // Invalidate cache
        if let Some(product_id) = product_id {
            if let Ok(Some(product_with_variants)) = self.repo.get_by_id(product_id).await {
                let _ = self.cache.delete_product(&product_with_variants.product.slug).await;
            }
        }

        Ok(())
    }
}
