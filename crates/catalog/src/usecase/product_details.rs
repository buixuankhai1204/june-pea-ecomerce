use crate::domain::cache::CatalogCache;
use crate::domain::catalog_repository::CatalogRepository;
use crate::domain::model::ProductWithVariants;
use shared::AppError;
use std::sync::Arc;
use tokio::spawn;

pub struct GetProductUsecase {
    repo: Arc<dyn CatalogRepository>,
    cache: Arc<dyn CatalogCache>,
}

impl GetProductUsecase {
    pub(crate) fn new(repo: Arc<dyn CatalogRepository>, cache: Arc<dyn CatalogCache>) -> Self {
        Self { repo, cache }
    }

    pub(crate) async fn execute(&self, slug: &str) -> Result<ProductWithVariants, AppError> {
        if let Ok(Some(cached_product)) = self.cache.get_product(slug).await {
            tracing::info!("Cache hit for slug: {}", slug);
            return Ok(cached_product);
        }

        tracing::info!("Cache miss for slug: {}. Fetching from DB...", slug);
        let product = self
            .repo
            .get_by_slug(slug)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product {} not found", slug)))?;

        let cache_clone = self.cache.clone();
        let slug_clone = slug.to_string();
        let product_clone = product.clone();

        spawn(async move {
            if let Err(e) = cache_clone.set_product(&slug_clone, &product_clone).await {
                tracing::error!("Failed to update cache: {:?}", e);
            }
        });

        Ok(product)
    }

    #[allow(dead_code)]
    fn catalog_repository(&self) -> Arc<dyn CatalogRepository> {
        self.repo.clone()
    }
}
