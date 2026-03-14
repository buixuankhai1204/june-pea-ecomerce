use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Newtype IDs ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VariantId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

// ─── Auth ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub email: String,
}

// ─── Catalog ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub category_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub base_price: Decimal,
    pub sale_price: Option<Decimal>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductWithVariants {
    pub product: Product,
    pub variants: Vec<ProductVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub category_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub category_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedProducts {
    pub items: Vec<Product>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVariantRequest {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub base_price: Decimal,
    pub sale_price: Option<Decimal>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVariantRequest {
    pub sku: String,
    pub name: String,
    pub base_price: Decimal,
    pub sale_price: Option<Decimal>,
    pub attributes: serde_json::Value,
}

// ─── Ordering ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Cancelled,
    Completed,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Completed => write!(f, "Completed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub status: OrderStatus,
    pub total: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderItem {
    pub variant_id: Uuid,
    pub quantity: i32,
    pub unit_price: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    pub customer_id: Option<Uuid>,
    pub items: Vec<NewOrderItem>,
    pub coupon_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderResponse {
    pub order_id: Uuid,
}

// ─── Marketing ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coupon {
    pub id: Uuid,
    pub code: String,
    pub discount_amount: i64,
    pub max_uses: i32,
    pub current_uses: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCouponRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCouponResponse {
    pub code: String,
    pub discount_amount: i64,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCouponRequest {
    pub code: String,
    pub discount_amount: i64,
    pub max_uses: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
}

// ─── Inventory ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockUpdate {
    pub variant_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockResponse {
    pub variant_id: Uuid,
    pub quantity: i32,
}

// ─── API Error ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub success: bool,
    pub error: String,
}

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Validation Error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Server Error: {0}")]
    Server(String),
}

impl ApiError {
    pub fn user_message(&self) -> &str {
        match self {
            Self::Network(m) | Self::Unauthorized(m) | Self::NotFound(m) | Self::Validation(m) | Self::Conflict(m) | Self::Server(m) => m,
        }
    }
}
