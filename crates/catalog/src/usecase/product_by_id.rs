use std::sync::Arc;
use uuid::Uuid;
use crate::domain::catalog_repository::CatalogRepository;
use crate::domain::model::ProductWithVariants;
use shared::AppError;

pub struct GetProductByIdUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl GetProductByIdUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<ProductWithVariants, AppError> {
        let product = self.repo.get_by_id(id).await?;
        product.ok_or_else(|| AppError::NotFound("Product not found".to_string()))
    }
}
