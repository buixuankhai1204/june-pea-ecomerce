use crate::domain::catalog_repository::CatalogRepository;
use rust_decimal::Decimal;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateVariantUsecase {
    repo: Arc<dyn CatalogRepository>,
}

impl CreateVariantUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        product_id: Uuid,
        sku: String,
        name: String,
        base_price: Decimal,
        sale_price: Option<Decimal>,
        attributes: serde_json::Value,
    ) -> Result<Uuid, AppError> {
        let id = Uuid::new_v4();
        self.repo
            .create_variant(
                id, product_id, &sku, &name, base_price, sale_price, attributes,
            )
            .await?;
        Ok(id)
    }
}
