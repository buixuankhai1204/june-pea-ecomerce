use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub category_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<Uuid>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductWithVariants {
    pub product: Product,
    pub variants: Vec<ProductVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedProducts {
    pub items: Vec<ProductWithVariants>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub category_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: Option<String>,
    pub parent_id: Option<Uuid>,
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
pub struct UpdateProductRequest {
    pub category_id: Uuid,
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVariantRequest {
    pub sku: String,
    pub name: String,
    pub base_price: Decimal,
    pub sale_price: Option<Decimal>,
    pub attributes: serde_json::Value,
}

// ─── Analytics ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenuePoint {
    pub label: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowStockItem {
    pub variant_id: Uuid,
    pub product_name: String,
    pub variant_name: String,
    pub current_stock: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub category_name: String,
    pub sales_percentage: f64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummary {
    pub today_orders: u32,
    pub total_revenue: i64,
    pub new_products_this_week: u32,
    pub stock_accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryNode {
    pub category: Category,
    pub children: Vec<CategoryNode>,
}

// ─── Identity ───

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StaffRole {
    Admin,
    Staff,
}

impl std::fmt::Display for StaffRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "Admin"),
            Self::Staff => write!(f, "Staff"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: Uuid,
    pub exp: usize,
    pub role: String,
}

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

#[derive(Debug, serde::Deserialize, Serialize, Clone, PartialEq)]
pub struct UpdateProfileRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StaffMember {
    pub id: Uuid,
    pub email: String,
    pub role: StaffRole,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub status: OrderStatus,
    pub total: i64,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderItem {
    pub variant_id: Uuid,
    pub quantity: i32,
    pub unit_price: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOrderRequest {
    pub customer_id: Option<Uuid>,
    pub items: Vec<NewOrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderResponse {
    pub order_id: Uuid,
}

// ─── Payment ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Expired,
    Refunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Paid => write!(f, "Paid"),
            Self::Failed => write!(f, "Failed"),
            Self::Expired => write!(f, "Expired"),
            Self::Refunded => write!(f, "Refunded"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invoice {
    pub id: Uuid,
    pub order_id: Uuid,
    pub customer_email: String,
    pub amount: Decimal,
    pub total: i64,
    pub status: PaymentStatus,
    pub issued_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntentView {
    pub payment_id: Uuid,
    pub order_id: Uuid,
    pub amount: i64,
    pub payment_url: String,
    pub qr_svg: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub status: PaymentStatus,
    pub txn_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVnPayQrRequest {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateCouponRequest {
    pub code: String,
    pub discount_amount: i64,
    pub max_uses: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplyCategoryDiscountRequest {
    pub category_id: Uuid,
    pub percent: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateOrderNoteRequest {
    pub note: String,
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

// ─── Suppliers ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Supplier {
    pub id: Uuid,
    pub name: String,
    pub contact: String,
    pub location: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub contact: String,
    pub location: String,
}

// ─── Memberships ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Membership {
    pub id: Uuid,
    pub email: String,
    pub tier: String,
    pub points: i32,
    pub total_spent: Decimal,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipSummary {
    pub total_members: i32,
    pub tier_counts: std::collections::HashMap<String, i32>,
}

// ─── Errors ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Server error: {0}")]
    Server(String),
}

impl ApiError {
    pub fn user_message(&self) -> &str {
        match self {
            Self::Network(m)
            | Self::Unauthorized(m)
            | Self::NotFound(m)
            | Self::Validation(m)
            | Self::Conflict(m)
            | Self::Server(m) => m,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub error: String,
}
