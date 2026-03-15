use crate::domain::catalog_repository::CatalogRepository;
use rust_decimal::Decimal;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateVariantUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl UpdateVariantUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: Uuid,
        sku: String,
        name: String,
        base_price: Decimal,
        sale_price: Option<Decimal>,
        attributes: serde_json::Value,
    ) -> Result<(), AppError> {
        self.repo
            .update_variant(id, &sku, &name, base_price, sale_price, attributes)
            .await
    }
}
