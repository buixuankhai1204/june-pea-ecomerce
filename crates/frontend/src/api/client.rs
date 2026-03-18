pub use crate::api::types::*;
#[cfg(not(target_arch = "wasm32"))]
use dotenv::dotenv;
use gloo_net::http::{Request, RequestBuilder};
use gloo_storage::{LocalStorage, Storage};
use serde::de::DeserializeOwned;
use serde::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;
use uuid::Uuid;

const TOKEN_KEY: &str = "june_pea_token";
const DEFAULT_BASE_URL: &str = "https://june-pea-backend-production.up.railway.app";

fn base_url() -> String {
    let resolved = resolve_base_url_impl();
    println!("Using base url: {}", resolved);
    resolved
}

#[cfg(target_arch = "wasm32")]
fn resolve_base_url_impl() -> String {
    option_env!("API_URL")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_base_url_impl() -> String {
    static DOTENV_ONCE: OnceLock<()> = OnceLock::new();
    DOTENV_ONCE.get_or_init(|| {
        let _ = dotenv();
    });

    env::var("API_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            option_env!("API_URL")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

pub fn get_token() -> Option<String> {
    LocalStorage::get(TOKEN_KEY).ok()
}

pub fn set_token(token: &str) {
    let _ = LocalStorage::set(TOKEN_KEY, token.to_string());
}

pub fn clear_token() {
    LocalStorage::delete(TOKEN_KEY);
}

fn apply_headers(req: RequestBuilder) -> RequestBuilder {
    let req = req
        .header("Content-Type", "application/json")
        .header("Accept", "application/json");

    if let Some(token) = get_token() {
        req.header("Authorization", &format!("Bearer {}", token))
    } else {
        req
    }
}

async fn parse_response<T: DeserializeOwned>(
    resp: gloo_net::http::Response,
) -> Result<T, ApiError> {
    let status = resp.status();
    if (200..300).contains(&status) {
        resp.json::<T>()
            .await
            .map_err(|e| ApiError::Network(format!("Failed to parse response: {}", e)))
    } else {
        let body = match resp.json::<ApiErrorBody>().await {
            Ok(b) => b.error,
            Err(_) => "Unknown error".to_string(),
        };

        Err(match status {
            401 => ApiError::Unauthorized(body),
            404 => ApiError::NotFound(body),
            400 => ApiError::Validation(body),
            409 => ApiError::Conflict(body),
            _ => ApiError::Server(body),
        })
    }
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = format!("{}{}", base_url(), path);
    let req = apply_headers(Request::get(&url));
    let resp = req
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;
    parse_response::<T>(resp).await
}

pub async fn post<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> Result<T, ApiError> {
    let url = format!("{}{}", base_url(), path);
    let builder = apply_headers(Request::post(&url));
    let request = builder
        .json(body)
        .map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = request
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;
    parse_response::<T>(resp).await
}

pub async fn patch<T: DeserializeOwned, B: Serialize>(path: &str, body: &B) -> Result<T, ApiError> {
    let url = format!("{}{}", base_url(), path);
    let builder = apply_headers(Request::patch(&url));
    let request = builder
        .json(body)
        .map_err(|e| ApiError::Network(e.to_string()))?;
    let resp = request
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;
    parse_response::<T>(resp).await
}

pub async fn delete<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = format!("{}{}", base_url(), path);
    let req = apply_headers(Request::delete(&url));
    let resp = req
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;
    parse_response::<T>(resp).await
}

pub mod identity {
    use super::*;

    pub async fn register(req: RegisterRequest) -> Result<User, ApiError> {
        post("/api/v1/auth/register", &req).await
    }

    pub async fn login(req: LoginRequest) -> Result<LoginResponse, ApiError> {
        post("/api/v1/auth/login", &req).await
    }

    pub async fn get_me() -> Result<User, ApiError> {
        get("/api/v1/auth/me").await
    }

    pub async fn update_profile(req: UpdateProfileRequest) -> Result<User, ApiError> {
        patch("/api/v1/auth/profile", &req).await
    }

    pub async fn list_users() -> Result<Vec<User>, ApiError> {
        get("/api/v1/auth/users").await
    }

    pub async fn list_staff() -> Result<Vec<StaffMember>, ApiError> {
        get("/api/v1/identity/staff").await
    }

    pub async fn create_staff(email: &str, password: &str) -> Result<serde_json::Value, ApiError> {
        post("/api/v1/identity/staff", &serde_json::json!({
            "email": email,
            "password": password
        })).await
    }

    pub async fn change_password(req: ChangePasswordRequest) -> Result<bool, ApiError> {
        post("/api/v1/auth/change-password", &req).await
    }

    pub async fn delete_user(id: Uuid) -> Result<bool, ApiError> {
        delete(&format!("/api/v1/auth/users/{}", id)).await
    }
}

pub mod catalog {
    use super::*;

    pub async fn list_products(page: i64, page_size: i64) -> Result<PaginatedProducts, ApiError> {
        get(&format!(
            "/api/v1/catalog/products?page={}&page_size={}",
            page, page_size
        ))
        .await
    }

    pub async fn get_product(slug: &str) -> Result<ProductWithVariants, ApiError> {
        get(&format!("/api/v1/catalog/products/slug/{}", slug)).await
    }

    pub async fn get_product_by_id(id: Uuid) -> Result<ProductWithVariants, ApiError> {
        get(&format!("/api/v1/catalog/products/{}", id)).await
    }

    pub async fn list_categories() -> Result<Vec<Category>, ApiError> {
        get("/api/v1/catalog/categories").await
    }

    pub async fn create_category(req: CreateCategoryRequest) -> Result<bool, ApiError> {
        post::<bool, _>("/api/v1/catalog/categories", &req).await
    }

    pub async fn create_product(req: CreateProductRequest) -> Result<bool, ApiError> {
        post::<bool, _>("/api/v1/catalog/products", &req).await
    }

    pub async fn update_product(id: Uuid, req: UpdateProductRequest) -> Result<bool, ApiError> {
        patch::<bool, _>(&format!("/api/v1/catalog/products/{}", id), &req).await
    }

    pub async fn delete_product(id: Uuid) -> Result<bool, ApiError> {
        delete::<bool>(&format!("/api/v1/catalog/products/{}", id)).await
    }

    pub async fn delete_category(id: Uuid) -> Result<bool, ApiError> {
        delete::<bool>(&format!("/api/v1/catalog/categories/{}", id)).await
    }

    pub async fn create_variant(req: CreateVariantRequest) -> Result<Uuid, ApiError> {
        post::<Uuid, _>("/api/v1/catalog/variants", &req).await
    }

    pub async fn update_variant(id: Uuid, req: UpdateVariantRequest) -> Result<bool, ApiError> {
        patch::<bool, _>(&format!("/api/v1/catalog/variants/{}", id), &req).await
    }

    pub async fn delete_variant(id: Uuid) -> Result<bool, ApiError> {
        delete::<bool>(&format!("/api/v1/catalog/variants/{}", id)).await
    }

    pub async fn search_products(query: &str) -> Result<Vec<ProductWithVariants>, ApiError> {
        get(&format!("/api/v1/catalog/search?q={}", query)).await
    }

    pub async fn get_category_tree() -> Result<Vec<CategoryNode>, ApiError> {
        get("/api/v1/catalog/categories/tree").await
    }
}

pub mod inventory {
    use super::*;

    pub async fn get_stock(variant_id: Uuid) -> Result<serde_json::Value, ApiError> {
        get(&format!("/api/v1/inventory/stock/{}", variant_id)).await
    }

    pub async fn update_stock(req: StockUpdate) -> Result<bool, ApiError> {
        post::<bool, _>("/api/v1/inventory/update-stock", &req).await
    }

    pub async fn list_all_stocks() -> Result<Vec<StockResponse>, ApiError> {
        get("/api/v1/inventory/list-all").await
    }

    pub async fn check_low_stock_alerts(threshold: i32) -> Result<Vec<StockResponse>, ApiError> {
        get(&format!("/api/v1/inventory/stock/low-alerts?threshold={}", threshold)).await
    }
}

pub mod ordering {
    use super::*;

    pub async fn place_order(req: PlaceOrderRequest) -> Result<PlaceOrderResponse, ApiError> {
        post("/api/v1/ordering/orders", &req).await
    }

    pub async fn update_order_status(id: Uuid, status: OrderStatus) -> Result<bool, ApiError> {
        patch::<bool, _>(
            &format!("/api/v1/ordering/orders/{}/status", id),
            &UpdateOrderStatusRequest { status },
        )
        .await
    }

    pub async fn list_orders(customer_id: Uuid) -> Result<Vec<Order>, ApiError> {
        get(&format!("/api/v1/ordering/orders/customer/{}", customer_id)).await
    }

    pub async fn list_all_orders() -> Result<Vec<Order>, ApiError> {
        get("/api/v1/ordering/orders").await
    }

    pub async fn update_order_note(id: Uuid, note: &str) -> Result<bool, ApiError> {
        patch::<bool, _>(
            &format!("/api/v1/ordering/orders/{}/note", id),
            &UpdateOrderNoteRequest { note: note.to_string() },
        )
        .await
    }

    pub async fn list_recent_orders(customer_id: Uuid) -> Result<Vec<Order>, ApiError> {
        get(&format!("/api/v1/ordering/orders/customer/{}/recent", customer_id)).await
    }
}

pub mod marketing {
    use super::*;

    pub async fn list_coupons() -> Result<Vec<Coupon>, ApiError> {
        get("/api/v1/marketing/coupons").await
    }

    pub async fn create_coupon(req: CreateCouponRequest) -> Result<Coupon, ApiError> {
        post("/api/v1/marketing/coupons", &req).await
    }

    pub async fn deactivate_coupon(code: &str) -> Result<serde_json::Value, ApiError> {
        patch(&format!("/api/v1/marketing/coupons/{}/deactivate", code), &serde_json::json!({})).await
    }

    pub async fn delete_coupon(code: &str) -> Result<serde_json::Value, ApiError> {
        delete(&format!("/api/v1/marketing/coupons/{}", code)).await
    }

    pub async fn validate_coupon(
        req: ValidateCouponRequest,
    ) -> Result<ValidateCouponResponse, ApiError> {
        post("/api/v1/marketing/coupons/validate", &req).await
    }

    pub async fn apply_category_discount(req: ApplyCategoryDiscountRequest) -> Result<bool, ApiError> {
        post("/api/v1/marketing/discounts/category", &req).await
    }
}

pub mod payment {
    use super::*;

    pub async fn create_vnpay_qr(req: CreateVnPayQrRequest) -> Result<PaymentIntentView, ApiError> {
        post("/api/v1/payment/vnpay/qr", &req).await
    }

    pub async fn get_payment_status(order_id: Uuid) -> Result<PaymentIntentView, ApiError> {
        get(&format!("/api/v1/payment/orders/{}", order_id)).await
    }

    pub async fn refund_payment(order_id: Uuid) -> Result<bool, ApiError> {
        post::<bool, _>(&format!("/api/v1/payment/orders/{}/refund", order_id), &serde_json::json!({})).await
    }
}

pub mod suppliers {
    use super::*;

    pub async fn list_suppliers() -> Result<Vec<Supplier>, ApiError> {
        get("/api/v1/inventory/suppliers").await
    }

    pub async fn create_supplier(req: CreateSupplierRequest) -> Result<Supplier, ApiError> {
        post("/api/v1/inventory/suppliers", &req).await
    }

    pub async fn delete_supplier(id: Uuid) -> Result<serde_json::Value, ApiError> {
        delete(&format!("/api/v1/inventory/suppliers/{}", id)).await
    }
}

pub mod memberships {
    use super::*;

    pub async fn list_memberships() -> Result<Vec<Membership>, ApiError> {
        get("/api/v1/auth/memberships").await
    }

    pub async fn get_membership_summary() -> Result<MembershipSummary, ApiError> {
        get("/api/v1/auth/memberships/summary").await
    }
}

pub mod analytics {
    use super::*;

    pub async fn get_dashboard_summary() -> Result<DashboardSummary, ApiError> {
        get("/api/v1/analytics/dashboard-summary").await
    }

    pub async fn get_revenue_analytics() -> Result<Vec<RevenuePoint>, ApiError> {
        get("/api/v1/analytics/revenue-analytics").await
    }

    pub async fn get_low_stock_items() -> Result<Vec<LowStockItem>, ApiError> {
        get("/api/v1/analytics/low-stock-items").await
    }

    pub async fn get_category_breakdown() -> Result<Vec<CategoryBreakdown>, ApiError> {
        get("/api/v1/analytics/category-breakdown").await
    }

    pub async fn get_recent_invoices() -> Result<Vec<Invoice>, ApiError> {
        get("/api/v1/analytics/recent-invoices").await
    }

    pub async fn export_order_report() -> Result<String, ApiError> {
        get("/api/v1/analytics/export-orders").await
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn runtime_env_variable_is_used_when_available() {
        std::env::set_var("API_URL", "https://env.test");
        assert_eq!(base_url(), "https://env.test");
        std::env::remove_var("API_URL");
    }
}
