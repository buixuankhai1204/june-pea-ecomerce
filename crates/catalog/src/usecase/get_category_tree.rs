use crate::domain::catalog_repository::CatalogRepository;
use crate::domain::model::Category;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryNode {
    pub category: Category,
    pub children: Vec<CategoryNode>,
}

pub struct GetCategoryTreeUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl GetCategoryTreeUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<CategoryNode>, AppError> {
        self.build_tree(None).await
    }

    #[async_recursion::async_recursion]
    async fn build_tree(&self, parent_id: Option<Uuid>) -> Result<Vec<CategoryNode>, AppError> {
        let categories = self.repo.get_categories_by_parent(parent_id).await?;
        let mut nodes = Vec::new();

        for cat in categories {
            let children = self.build_tree(Some(cat.id)).await?;
            nodes.push(CategoryNode {
                category: cat,
                children,
            });
        }

        Ok(nodes)
    }
}
