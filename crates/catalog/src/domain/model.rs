use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub base_price: Decimal,
    pub sale_price: Option<Decimal>,
    pub attributes: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A composite DTO for the Storefront
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductWithVariants {
    pub product: Product,
    pub variants: Vec<ProductVariant>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedProducts {
    pub items: Vec<ProductWithVariants>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow, PartialEq)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
