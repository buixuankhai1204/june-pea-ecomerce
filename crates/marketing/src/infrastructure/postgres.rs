use crate::domain::{model::Coupon, repository::CouponRepository};
use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError, infrastructure::postgres::SqlxExecutor};
use sqlx::PgPool;

pub struct PostgresCouponRepository {
    pub pool: PgPool,
}

impl PostgresCouponRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CouponRepository for PostgresCouponRepository {
    async fn create_coupon(
        &self,
        exec: &mut dyn DbExecutor,
        coupon: &Coupon,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query(
            r#"
            INSERT INTO marketing.coupons (id, code, discount_amount, max_uses, current_uses, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(coupon.id)
        .bind(&coupon.code)
        .bind(coupon.discount_amount)
        .bind(coupon.max_uses)
        .bind(coupon.current_uses)
        .bind(coupon.is_active)
        .bind(coupon.created_at)
        .execute(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    async fn get_coupon_by_code(
        &self,
        exec: &mut dyn DbExecutor,
        code: &str,
    ) -> Result<Coupon, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let row = sqlx::query_as::<_, Coupon>(
            r#"
            SELECT id, code, discount_amount, max_uses, current_uses, is_active, created_at
            FROM marketing.coupons
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_one(&mut *executor.tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Coupon {} not found", code)),
            _ => AppError::InternalServerError,
        })?;

        Ok(row)
    }

    async fn update_coupon(
        &self,
        exec: &mut dyn DbExecutor,
        coupon: &Coupon,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query(
            r#"
            UPDATE marketing.coupons
            SET discount_amount = $1, max_uses = $2, current_uses = $3, is_active = $4
            WHERE id = $5
            "#,
        )
        .bind(coupon.discount_amount)
        .bind(coupon.max_uses)
        .bind(coupon.current_uses)
        .bind(coupon.is_active)
        .bind(coupon.id)
        .execute(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    async fn list_coupons(&self, exec: &mut dyn DbExecutor) -> Result<Vec<Coupon>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let rows = sqlx::query_as::<_, Coupon>(
            r#"
            SELECT id, code, discount_amount, max_uses, current_uses, is_active, created_at
            FROM marketing.coupons
            "#,
        )
        .fetch_all(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(rows)
    }

    async fn delete_coupon(
        &self,
        exec: &mut dyn DbExecutor,
        id: uuid::Uuid,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query("DELETE FROM marketing.coupons WHERE id = $1")
            .bind(id)
            .execute(&mut *executor.tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound(format!("Coupon {} not found", id)),
                _ => AppError::InternalServerError,
            })?;

        Ok(())
    }

    async fn apply_category_discount(
        &self,
        _exec: &mut dyn DbExecutor,
        category_id: uuid::Uuid,
        percent: i32,
    ) -> Result<(), AppError> {
        let factor = (100 - percent) as f64 / 100.0;
        
        // Update sale_price based on base_price and the percentage discount for all products in the category
        sqlx::query(
            r#"
            UPDATE catalog.product_variants
            SET sale_price = (base_price * $1)
            FROM catalog.products
            WHERE catalog.product_variants.product_id = catalog.products.id
            AND catalog.products.category_id = $2
            "#
        )
        .bind(factor)
        .bind(category_id)
        .execute(&self.pool)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }
}
