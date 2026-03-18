use crate::domain::catalog_repository::CatalogRepository;
use crate::domain::model::{PaginatedProducts, ProductWithVariants};
use shared::AppError;
use std::sync::Arc;

pub struct ListProductsUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl ListProductsUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, page: i64, page_size: i64) -> Result<PaginatedProducts, AppError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 100);
        let offset = (page - 1) * page_size;

        let items = self.repo.list_all(page_size, offset).await?;
        let total = self.repo.count_all().await?;

        Ok(PaginatedProducts {
            items,
            total,
            page,
            page_size,
        })
    }
}
