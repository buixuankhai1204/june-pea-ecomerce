use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::sync::Arc;

pub struct ListCategoriesUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl ListCategoriesUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<crate::domain::model::Category>, AppError> {
        self.repo.list_categories().await
    }
}
