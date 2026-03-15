use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateCategoryUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl CreateCategoryUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        name: String,
        slug: Option<String>,
        parent_id: Option<Uuid>,
    ) -> Result<Uuid, AppError> {
        let id = Uuid::new_v4();
        let slug = slug.unwrap_or_else(|| name.to_lowercase().replace(" ", "-"));
        self.repo
            .create_category(id, &name, &slug, parent_id)
            .await?;
        Ok(id)
    }
}
