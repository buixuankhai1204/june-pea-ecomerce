use crate::usecase::{
    get_dashboard_summary::GetDashboardSummaryUsecase,
    get_revenue_analytics::GetRevenueAnalyticsUsecase,
    get_low_stock_items::GetLowStockItemsUsecase,
    get_category_breakdown::GetCategoryBreakdownUsecase,
    recent_invoices::GetRecentInvoicesUsecase,
    export_order_report::ExportOrderReportUsecase,
};
use axum::{extract::State, routing::get, Json, Router};
use shared::error::AppError;
use std::sync::Arc;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AnalyticsUsecase {
    dashboard_summary: Arc<GetDashboardSummaryUsecase>,
    revenue_analytics: Arc<GetRevenueAnalyticsUsecase>,
    low_stock_items: Arc<GetLowStockItemsUsecase>,
    category_breakdown: Arc<GetCategoryBreakdownUsecase>,
    recent_invoices: Arc<GetRecentInvoicesUsecase>,
    export_order_report: Arc<ExportOrderReportUsecase>,
}

impl AnalyticsUsecase {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            dashboard_summary: Arc::new(GetDashboardSummaryUsecase::new(pool.clone())),
            revenue_analytics: Arc::new(GetRevenueAnalyticsUsecase::new(pool.clone())),
            low_stock_items: Arc::new(GetLowStockItemsUsecase::new(pool.clone())),
            category_breakdown: Arc::new(GetCategoryBreakdownUsecase::new(pool.clone())),
            recent_invoices: Arc::new(GetRecentInvoicesUsecase::new(pool.clone())),
            export_order_report: Arc::new(ExportOrderReportUsecase::new(pool)),
        }
    }
}

pub fn init() -> Router<AnalyticsUsecase> {
    Router::new()
        .route("/dashboard-summary", get(get_dashboard_summary_handler))
        .route("/revenue-analytics", get(get_revenue_analytics_handler))
        .route("/low-stock", get(get_low_stock_handler))
        .route("/category-breakdown", get(get_category_breakdown_handler))
        .route("/recent-invoices", get(get_recent_invoices_handler))
        .route("/export-orders", get(export_orders_handler))
}

async fn get_dashboard_summary_handler(
    State(state): State<AnalyticsUsecase>,
) -> Result<Json<crate::domain::model::DashboardSummary>, AppError> {
    Ok(Json(state.dashboard_summary.execute().await?))
}

async fn get_revenue_analytics_handler(
    State(state): State<AnalyticsUsecase>,
) -> Result<Json<Vec<crate::domain::model::RevenuePoint>>, AppError> {
    Ok(Json(state.revenue_analytics.execute().await?))
}

async fn get_low_stock_handler(
    State(state): State<AnalyticsUsecase>,
) -> Result<Json<Vec<crate::domain::model::LowStockItem>>, AppError> {
    Ok(Json(state.low_stock_items.execute().await?))
}

async fn get_category_breakdown_handler(
    State(state): State<AnalyticsUsecase>,
) -> Result<Json<Vec<crate::domain::model::CategoryBreakdown>>, AppError> {
    Ok(Json(state.category_breakdown.execute().await?))
}

async fn get_recent_invoices_handler(
    State(state): State<AnalyticsUsecase>,
) -> Result<Json<Vec<crate::usecase::recent_invoices::Invoice>>, AppError> {
    Ok(Json(state.recent_invoices.execute().await?))
}

async fn export_orders_handler(
    State(state): State<AnalyticsUsecase>,
) -> Result<axum::response::Response, AppError> {
    let csv = state.export_order_report.execute().await?;
    Ok(axum::response::Response::builder()
        .header("Content-Type", "text/csv")
        .header("Content-Disposition", "attachment; filename=\"orders.csv\"")
        .body(axum::body::Body::from(csv))
        .map_err(|_| AppError::InternalServerError)?)
}
