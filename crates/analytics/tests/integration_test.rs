//! Integration tests for analytics. This service has no tables of its own: every query reads
//! the ordering, catalog, inventory and identify schemas directly, so what's under test is
//! those queries against a real, fully migrated Postgres with data from several services.
//!
//! `#[sqlx::test]` gives each test its own freshly migrated database on the server named by
//! `DATABASE_URL`. The migrations seed a few rows, so tests clear what they count first.
//!
//! Run: `cargo test -p analytics --test integration_test` (needs DATABASE_URL)

use analytics::routes::{init, AnalyticsUsecase};
use analytics::usecase::{
    export_order_report::ExportOrderReportUsecase,
    get_dashboard_summary::GetDashboardSummaryUsecase, get_low_stock_items::GetLowStockItemsUsecase,
    recent_invoices::GetRecentInvoicesUsecase,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// the seeded customer1@example.com
const CUSTOMER1: &str = "22222222-2222-2222-2222-222222222222";

async fn clear_orders(pool: &PgPool) {
    sqlx::query("TRUNCATE ordering.orders CASCADE").execute(pool).await.unwrap();
}

/// `days_ago` days before now
async fn order(pool: &PgPool, customer: Option<&str>, status: &str, total: i64, days_ago: i32) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO ordering.orders (id, customer_id, status, total, created_at)
         VALUES ($1, $2::uuid, $3, $4, NOW() - make_interval(days => $5))",
    )
    .bind(id)
    .bind(customer)
    .bind(status)
    .bind(total)
    .bind(days_ago)
    .execute(pool)
    .await
    .unwrap();
    id
}

#[sqlx::test(migrations = "../../migrations")]
async fn revenue_is_the_total_of_completed_orders_only(pool: PgPool) {
    clear_orders(&pool).await;
    order(&pool, None, "completed", 100_000, 0).await;
    order(&pool, None, "completed", 50_000, 3).await;
    order(&pool, None, "pending", 999_999, 0).await;
    order(&pool, None, "cancelled", 7, 0).await;

    let summary = GetDashboardSummaryUsecase::new(Arc::new(pool)).execute().await.unwrap();

    assert_eq!(summary.total_revenue, 150_000);
}

#[sqlx::test(migrations = "../../migrations")]
async fn revenue_is_zero_when_nothing_is_completed(pool: PgPool) {
    clear_orders(&pool).await;
    order(&pool, None, "pending", 5_000, 0).await;

    let summary = GetDashboardSummaryUsecase::new(Arc::new(pool)).execute().await.unwrap();

    assert_eq!(summary.total_revenue, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn today_counts_orders_of_the_last_day_whatever_their_status(pool: PgPool) {
    clear_orders(&pool).await;
    order(&pool, None, "pending", 1, 0).await;
    order(&pool, None, "cancelled", 1, 0).await;
    order(&pool, None, "completed", 1, 3).await;

    let summary = GetDashboardSummaryUsecase::new(Arc::new(pool)).execute().await.unwrap();

    assert_eq!(summary.today_orders, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn new_products_counts_the_last_seven_days(pool: PgPool) {
    sqlx::query("TRUNCATE catalog.products CASCADE").execute(&pool).await.unwrap();
    for (slug, days_ago) in [("fresh-1", 0), ("fresh-2", 6), ("old", 30)] {
        sqlx::query(
            "INSERT INTO catalog.products (id, category_id, name, slug, description, created_at)
             VALUES ($1, '44444444-4444-4444-4444-444444444444', $2, $2, '', NOW() - make_interval(days => $3))",
        )
        .bind(Uuid::new_v4())
        .bind(slug)
        .bind(days_ago)
        .execute(&pool)
        .await
        .unwrap();
    }

    let summary = GetDashboardSummaryUsecase::new(Arc::new(pool)).execute().await.unwrap();

    assert_eq!(summary.new_products_this_week, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn low_stock_lists_variants_under_twenty_fewest_first_with_their_names(pool: PgPool) {
    for (variant, quantity) in [
        ("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", 19),
        ("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb", 5),
        ("cccccccc-cccc-cccc-cccc-cccccccccccc", 20), // exactly 20 is not low
    ] {
        sqlx::query("UPDATE inventory.stock SET quantity = $1 WHERE variant_id = $2::uuid")
            .bind(quantity)
            .bind(variant)
            .execute(&pool)
            .await
            .unwrap();
    }

    let low = GetLowStockItemsUsecase::new(Arc::new(pool)).execute().await.unwrap();

    let seen: Vec<_> = low.iter().map(|i| (i.variant_name.as_str(), i.current_stock)).collect();
    assert_eq!(seen, vec![("Basic T-Shirt - White - M", 5), ("Basic T-Shirt - Black - L", 19)]);
    assert_eq!(low[0].product_name, "Basic T-Shirt");
}

#[sqlx::test(migrations = "../../migrations")]
async fn recent_invoices_are_the_ten_newest_with_the_customers_email_or_guest(pool: PgPool) {
    clear_orders(&pool).await;
    let mut newest = None;
    for days_ago in (0..12).rev() {
        // customer for the newest order, guests for the rest
        let customer = (days_ago == 0).then_some(CUSTOMER1);
        newest = Some(order(&pool, customer, "completed", 1_000 + days_ago as i64, days_ago).await);
    }

    let invoices = GetRecentInvoicesUsecase::new(Arc::new(pool)).execute().await.unwrap();

    assert_eq!(invoices.len(), 10);
    assert_eq!(invoices[0].order_id, newest.unwrap());
    assert_eq!(invoices[0].customer_email, "customer1@example.com");
    assert_eq!(invoices[1].customer_email, "Guest");
    assert!(invoices.windows(2).all(|w| w[0].created_at >= w[1].created_at));
}

#[sqlx::test(migrations = "../../migrations")]
async fn export_is_a_csv_with_a_header_and_every_order_newest_first(pool: PgPool) {
    clear_orders(&pool).await;
    let older = order(&pool, None, "pending", 2_000, 2).await;
    let newer = order(&pool, Some(CUSTOMER1), "completed", 3_000, 0).await;

    let csv = ExportOrderReportUsecase::new(Arc::new(pool)).execute().await.unwrap();

    let lines: Vec<_> = csv.lines().collect();
    assert_eq!(lines[0], "order_id,customer_id,status,total,created_at");
    assert_eq!(lines.len(), 3);
    assert!(lines[1].starts_with(&format!("{newer},{CUSTOMER1},completed,3000,")), "{}", lines[1]);
    assert!(lines[2].starts_with(&format!("{older},,pending,2000,")), "guest has no customer id: {}", lines[2]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_export_route_serves_it_as_a_csv_download(pool: PgPool) {
    let app = axum::Router::new().nest(
        "/api/v1/analytics",
        init().with_state(AnalyticsUsecase::new(Arc::new(pool))),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let res = reqwest::get(format!("{base}/api/v1/analytics/export-orders")).await.unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["content-type"], "text/csv");
    assert!(res.headers()["content-disposition"].to_str().unwrap().contains("orders.csv"));
    assert!(res.text().await.unwrap().starts_with("order_id,customer_id"));
}
