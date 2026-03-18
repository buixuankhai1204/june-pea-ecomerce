use crate::domain::catalog_repository::CatalogRepository;
use crate::domain::model::ProductWithVariants;
use shared::AppError;
use std::sync::Arc;

pub struct SearchProductsUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl SearchProductsUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, query: &str, limit: i64, offset: i64) -> Result<Vec<ProductWithVariants>, AppError> {
        self.repo.search_products(query, limit, offset).await
    }
}
