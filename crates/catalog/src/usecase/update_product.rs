use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateProductUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl UpdateProductUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        name: String,
        slug: Option<String>,
        category_id: Uuid,
        description: Option<String>,
    ) -> Result<(), AppError> {
        let slug = slug.unwrap_or_else(|| name.to_lowercase().replace(" ", "-"));
        self.repo
            .update_product(id, category_id, &name, &slug, description.as_deref())
            .await
    }
}
