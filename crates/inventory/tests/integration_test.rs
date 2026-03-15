use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use inventory::{
    infrastructure::persistence::postgres::PostgresInventoryRepository,
    usecase::{
        decrease_stock::DecreaseStockUsecase, get_stock::GetStockUsecase,
        increase_stock::IncreaseStockUsecase, update_stock::UpdateStockUsecase,
    },
};
use shared::{error::AppError, infrastructure::postgres::PostgresUnitOfWork};

struct TestContext {
    decrease_stock: DecreaseStockUsecase,
    increase_stock: IncreaseStockUsecase,
    get_stock: GetStockUsecase,
    update_stock: UpdateStockUsecase,
}

impl TestContext {
    fn new(pool: PgPool) -> Self {
        let repo = Arc::new(PostgresInventoryRepository);
        let uow = Arc::new(PostgresUnitOfWork::new(pool.clone()));

        Self {
            decrease_stock: DecreaseStockUsecase::new(repo.clone(), uow.clone()),
            increase_stock: IncreaseStockUsecase::new(repo.clone(), uow.clone()),
            get_stock: GetStockUsecase::new(repo.clone(), uow.clone()),
            update_stock: UpdateStockUsecase::new(repo, uow),
        }
    }
}

// Helper to construct some stock data
async fn inject_stock(pool: &PgPool, variant_id: Uuid, quantity: i32) {
    let category_id = Uuid::new_v4();
    let product_id = Uuid::new_v4();

    sqlx::query("INSERT INTO catalog.categories (id, name, slug) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(category_id)
        .bind("Test Category")
        .bind(format!("test-cat-{}", category_id))
        .execute(pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO catalog.products (id, category_id, name, slug) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
        .bind(product_id)
        .bind(category_id)
        .bind("Test Product")
        .bind(format!("test-prod-{}", product_id))
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO catalog.product_variants (id, product_id, sku, name, attributes, base_price) 
        VALUES ($1, $2, $3, $4, '{}'::jsonb, $5) ON CONFLICT DO NOTHING",
    )
    .bind(variant_id)
    .bind(product_id)
    .bind(format!("TEST-SKU-{}", variant_id))
    .bind("Test Variant")
    .bind(sqlx::types::Decimal::from(100))
    .execute(pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO inventory.stock (variant_id, quantity) VALUES ($1, $2)")
        .bind(variant_id)
        .bind(quantity)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_get_stock_works(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let variant_id = Uuid::new_v4();

    inject_stock(&pool, variant_id, 100).await;

    let stock_quantity = ctx.get_stock.execute(variant_id).await.unwrap();
    assert_eq!(stock_quantity, 100);
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_increase_stock_works(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let variant_id = Uuid::new_v4();

    inject_stock(&pool, variant_id, 100).await;

    // Increase stock
    ctx.increase_stock.execute(variant_id, 50).await.unwrap();

    // Validate
    let stock_quantity = ctx.get_stock.execute(variant_id).await.unwrap();
    assert_eq!(stock_quantity, 150);
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_decrease_stock_works(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let variant_id = Uuid::new_v4();

    inject_stock(&pool, variant_id, 100).await;

    // Decrease stock
    ctx.decrease_stock.execute(variant_id, 40).await.unwrap();

    // Validate
    let stock_quantity = ctx.get_stock.execute(variant_id).await.unwrap();
    assert_eq!(stock_quantity, 60);
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_decrease_stock_insufficient(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let variant_id = Uuid::new_v4();

    inject_stock(&pool, variant_id, 50).await;

    // Decrease stock by more than available
    let res = ctx.decrease_stock.execute(variant_id, 60).await;
    assert!(matches!(res, Err(AppError::Conflict(_))));

    // Validate stock remains unchanged
    let stock_quantity = ctx.get_stock.execute(variant_id).await.unwrap();
    assert_eq!(stock_quantity, 50);
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_update_stock_works(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let variant_id = Uuid::new_v4();

    inject_stock(&pool, variant_id, 100).await;

    // Update to absolute value
    ctx.update_stock.execute(variant_id, 500).await.unwrap();

    // Validate
    let stock_quantity = ctx.get_stock.execute(variant_id).await.unwrap();
    assert_eq!(stock_quantity, 500);
}
