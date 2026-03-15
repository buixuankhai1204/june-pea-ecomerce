use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use marketing::{
    infrastructure::postgres::PostgresCouponRepository,
    usecase::{
        create_coupon::CreateCouponUsecase, delete_coupon::DeleteCouponUsecase,
        validate_coupon::ValidateCouponUsecase,
    },
};
use shared::{error::AppError, infrastructure::postgres::PostgresUnitOfWork};

struct TestContext {
    create_coupon: CreateCouponUsecase,
    validate_coupon: ValidateCouponUsecase,
    delete_coupon: DeleteCouponUsecase,
}

impl TestContext {
    fn new(pool: PgPool) -> Self {
        let repo = Arc::new(PostgresCouponRepository::new(pool.clone()));
        let uow = Arc::new(PostgresUnitOfWork::new(pool.clone()));

        Self {
            create_coupon: CreateCouponUsecase::new(repo.clone(), uow.clone()),
            validate_coupon: ValidateCouponUsecase::new(repo.clone(), uow.clone()),
            delete_coupon: DeleteCouponUsecase::new(repo, uow),
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_create_coupon_persists_in_db(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let code = "TEST20".to_string();
    let discount = 2000;

    let _coupon = ctx
        .create_coupon
        .execute(code.clone(), discount, 10)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let row = sqlx::query("SELECT discount_amount FROM marketing.coupons WHERE code = $1")
        .bind(&code)
        .fetch_one(&mut *tx)
        .await
        .unwrap();

    let fetched_discount: i64 = sqlx::Row::try_get(&row, "discount_amount").unwrap();
    assert_eq!(fetched_discount, discount);
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_validate_coupon_works(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let code = "TEST30".to_string();

    // Create coupon
    ctx.create_coupon
        .execute(code.clone(), 3000, 5)
        .await
        .unwrap();

    // Validate coupon
    let coupon = ctx.validate_coupon.execute(&code).await.unwrap();
    assert_eq!(coupon.discount_amount, 3000);
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_validate_nonexistent_coupon_fails(pool: PgPool) {
    let ctx = TestContext::new(pool);
    let result = ctx.validate_coupon.execute("NONEXISTENT").await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_delete_coupon_works(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());
    let code = format!("DELETE-{}", Uuid::new_v4());

    // 1. Create
    let _coupon = ctx
        .create_coupon
        .execute(code.clone(), 5000, 1)
        .await
        .unwrap();

    // 2. Delete
    ctx.delete_coupon.execute(&code).await.unwrap();

    // 3. Verify gone
    let result = ctx.validate_coupon.execute(&code).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}
