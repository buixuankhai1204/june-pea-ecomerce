use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_revenue: i64,
    pub today_orders: usize,
    pub new_products_this_week: usize,
    pub stock_accuracy: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenuePoint {
    pub label: String,
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryBreakdown {
    pub category_name: String,
    pub sales_percentage: f64,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LowStockItem {
    pub variant_id: uuid::Uuid,
    pub product_name: String,
    pub variant_name: String,
    pub current_stock: i32,
}
