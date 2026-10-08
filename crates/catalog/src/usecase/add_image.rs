use crate::domain::cache::CatalogCache;
use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct AddProductImageUsecase {
    repo: Arc<dyn CatalogRepository>,
    cache: Arc<dyn CatalogCache>,
}

impl AddProductImageUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>, cache: Arc<dyn CatalogCache>) -> Self {
        Self { repo, cache }
    }

    pub async fn execute(
        &self,
        product_id: Uuid,
        url: String,
        is_primary: bool,
        position: i32,
    ) -> Result<(), AppError> {
        self.repo
            .add_product_image(Uuid::new_v4(), product_id, &url, is_primary, position)
            .await?;

        // Invalidate cache
        if let Ok(Some(product_with_variants)) = self.repo.get_by_id(product_id).await {
            let _ = self.cache.delete_product(&product_with_variants.product.slug).await;
        }

        Ok(())
    }
}
