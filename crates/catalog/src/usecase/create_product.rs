use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateProductUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl CreateProductUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        name: String,
        slug: Option<String>,
        category_id: Uuid,
        description: Option<String>,
    ) -> Result<Uuid, AppError> {
        let id = Uuid::new_v4();
        let slug = slug.unwrap_or_else(|| name.to_lowercase().replace(" ", "-"));
        self.repo
            .create_product(id, category_id, &name, &slug, description.as_deref())
            .await?;
        Ok(id)
    }
}
