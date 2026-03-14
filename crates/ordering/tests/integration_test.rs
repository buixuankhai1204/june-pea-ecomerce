//! E2E tests for ordering usecases.
//!
//! Each test runs against a real Postgres database provisioned by `#[sqlx::test]`.
//! All migrations in `migrations/` are applied before each test, and the
//! database is dropped afterward — giving full isolation with no mocks.
//!
//! A `TxContext` helper wraps assertions inside a transaction that is
//! always rolled back, so read-backs inside the same test see the
//! committed data from the usecase without polluting other tests.
//!
//! Run: `cargo test -p ordering --test e2e_test`
//!   (requires DATABASE_URL in .env pointing to a running Postgres instance)

use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

use ordering::{
    domain::model::{NewOrderItem, OrderStatus},
    infrastructure::persistence::postgres::PostgresOrderRepository,
    usecase::{
        cancel_order::CancelOrderUsecase, get_order::GetOrderUsecase,
        place_order::PlaceOrderUsecase,
        update_order_status::UpdateOrderStatusUsecase,
    },
};
use shared::{error::AppError, infrastructure::postgres::PostgresUnitOfWork};

// ─────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────

/// Holds the real infrastructure wired together for a single test.
struct TestContext {
    place_order: PlaceOrderUsecase,
    cancel_order: CancelOrderUsecase,
    get_order: GetOrderUsecase,
    update_order_status: UpdateOrderStatusUsecase,
}

impl TestContext {
    fn new(pool: PgPool) -> Self {
        let repo = Arc::new(PostgresOrderRepository::new(pool.clone()));
        let uow = Arc::new(PostgresUnitOfWork::new(pool.clone()));

        Self {
            place_order: PlaceOrderUsecase::new(repo.clone(), uow.clone()),
            cancel_order: CancelOrderUsecase::new(repo.clone(), uow.clone()),
            get_order: GetOrderUsecase::new(repo.clone(), uow.clone()),
            update_order_status: UpdateOrderStatusUsecase::new(repo, uow),
        }
    }
}

// Seed a real user into identify.users so FK on orders.customer_id is valid.
async fn seed_customer(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO identify.users (id, email, password_hash, role)
         VALUES ($1, $2, 'hash', 'customer')",
    )
    .bind(id)
    .bind(format!("test-{}@example.com", id))
    .execute(pool)
    .await
    .expect("seed customer");
    id
}

// Seed a product variant so FK on order_items.variant_id is valid.
async fn seed_variant(pool: &PgPool) -> Uuid {
    let cat_id = Uuid::new_v4();
    let product_id = Uuid::new_v4();
    let variant_id = Uuid::new_v4();

    sqlx::query("INSERT INTO catalog.categories (id, name, slug) VALUES ($1, 'Cat', $2)")
        .bind(cat_id)
        .bind(format!("cat-{}", cat_id))
        .execute(pool)
        .await
        .expect("seed category");

    sqlx::query(
        "INSERT INTO catalog.products (id, category_id, name, slug)
         VALUES ($1, $2, 'Product', $3)",
    )
    .bind(product_id)
    .bind(cat_id)
    .bind(format!("prod-{}", product_id))
    .execute(pool)
    .await
    .expect("seed product");

    sqlx::query(
        "INSERT INTO catalog.product_variants (id, product_id, sku, name, attributes, base_price)
         VALUES ($1, $2, $3, 'Variant', '{}', 10.00)",
    )
    .bind(variant_id)
    .bind(product_id)
    .bind(format!("sku-{}", variant_id))
    .execute(pool)
    .await
    .expect("seed variant");

    variant_id
}

// ─────────────────────────────────────────────
// PlaceOrder E2E tests
// ─────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_place_order_persists_order_and_items(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let customer = seed_customer(&pool).await;
    let variant = seed_variant(&pool).await;

    let order_id = ctx
        .place_order
        .execute(
            Some(customer),
            vec![NewOrderItem {
                variant_id: variant,
                quantity: 2,
                unit_price: 500,
            }],
        )
        .await
        .unwrap();

    // Verify persisted data via a read-only transaction (always rolled back)
    let mut tx = pool.begin().await.unwrap();

    let row = sqlx::query("SELECT status, total FROM ordering.orders WHERE id = $1")
        .bind(order_id)
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    let status: String = row.try_get("status").unwrap();
    let total: i64 = row.try_get("total").unwrap();
    assert_eq!(status, "pending");
    assert_eq!(total, 1000); // 2 * 500

    let item_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ordering.order_items WHERE order_id = $1")
            .bind(order_id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    assert_eq!(item_count, 1);

    tx.rollback().await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_place_order_with_empty_items_fails(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let customer = seed_customer(&pool).await;

    let result = ctx.place_order.execute(Some(customer), vec![]).await;
    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_place_order_with_zero_quantity_fails(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let customer = seed_customer(&pool).await;
    let variant = seed_variant(&pool).await;

    let result = ctx
        .place_order
        .execute(
            Some(customer),
            vec![NewOrderItem {
                variant_id: variant,
                quantity: 0,
                unit_price: 500,
            }],
        )
        .await;

    assert!(matches!(result, Err(AppError::Validation(_))));
}

// ─────────────────────────────────────────────
// GetOrder E2E tests
// ─────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_get_order_returns_correct_data(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let customer = seed_customer(&pool).await;
    let variant = seed_variant(&pool).await;

    let order_id = ctx
        .place_order
        .execute(
            Some(customer),
            vec![NewOrderItem {
                variant_id: variant,
                quantity: 3,
                unit_price: 200,
            }],
        )
        .await
        .unwrap();

    let order = ctx.get_order.execute(order_id).await.unwrap();

    assert_eq!(order.id, order_id);
    assert_eq!(order.customer_id, Some(customer));
    assert_eq!(order.status, OrderStatus::Pending);
    assert_eq!(order.total, 600); // 3 * 200
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_get_order_not_found_returns_error(pool: PgPool) {
    let ctx = TestContext::new(pool);
    let result = ctx.get_order.execute(Uuid::new_v4()).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

// ─────────────────────────────────────────────
// CancelOrder E2E tests
// ─────────────────────────────────────────────

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_cancel_pending_order_sets_cancelled_status(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let customer = seed_customer(&pool).await;
    let variant = seed_variant(&pool).await;
    let order_id = ctx
        .place_order
        .execute(
            Some(customer),
            vec![NewOrderItem {
                variant_id: variant,
                quantity: 1,
                unit_price: 100,
            }],
        )
        .await
        .unwrap();

    ctx.cancel_order.execute(order_id).await.unwrap();

    let fetched = ctx.get_order.execute(order_id).await.unwrap();
    assert_eq!(fetched.status, OrderStatus::Cancelled);

    // 4. Update to Completed
    ctx.update_order_status.execute(order_id, OrderStatus::Completed).await.unwrap();
    let fetched = ctx.get_order.execute(order_id).await.unwrap();
    assert_eq!(fetched.status, OrderStatus::Completed);
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_cancel_already_cancelled_order_fails(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let customer = seed_customer(&pool).await;
    let variant = seed_variant(&pool).await;

    let order_id = ctx
        .place_order
        .execute(
            Some(customer),
            vec![NewOrderItem {
                variant_id: variant,
                quantity: 1,
                unit_price: 100,
            }],
        )
        .await
        .unwrap();

    ctx.cancel_order.execute(order_id).await.unwrap();

    // Domain rule: cannot cancel a non-Pending order
    let result = ctx.cancel_order.execute(order_id).await;
    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_cancel_nonexistent_order_returns_not_found(pool: PgPool) {
    let ctx = TestContext::new(pool);
    let result = ctx.cancel_order.execute(Uuid::new_v4()).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}
