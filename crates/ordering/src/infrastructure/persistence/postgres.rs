use crate::domain::{
    model::{Order, OrderItem, OrderStatus},
    repository::OrderRepository,
};
use crate::infrastructure::persistence::reflect::order_row::OrderRow;

use async_trait::async_trait;
use shared::{database::DbExecutor, error::AppError, infrastructure::postgres::SqlxExecutor};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PostgresOrderRepository {
    pub pool: PgPool,
}

impl PostgresOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrderRepository for PostgresOrderRepository {
    async fn create_order(
        &self,
        exec: &mut dyn DbExecutor,
        order: &Order,
        items: &[OrderItem],
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);
        let status = status_to_str(&order.status);

        sqlx::query(
            r#"
            INSERT INTO ordering.orders (id, customer_id, status, total, note, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(order.id)
        .bind(order.customer_id)
        .bind(status)
        .bind(order.total)
        .bind(order.note.as_ref())
        .bind(order.created_at)
        .execute(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        for item in items {
            sqlx::query(
                r#"
                INSERT INTO ordering.order_items (id, order_id, variant_id, quantity, unit_price)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(item.id)
            .bind(item.order_id)
            .bind(item.variant_id)
            .bind(item.quantity)
            .bind(item.unit_price)
            .execute(&mut *executor.tx)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        }

        Ok(())
    }

    async fn get_order_by_id(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
    ) -> Result<Order, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let row = sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT id, customer_id, status, total, note, created_at
            FROM ordering.orders
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut *executor.tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::NotFound(format!("Order {} not found", id)),
            _ => AppError::InternalServerError,
        })?;

        Ok(row.into())
    }

    async fn update_order_status(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query(
            r#"
            UPDATE ordering.orders SET status = $1 WHERE id = $2
            "#,
        )
        .bind(status_to_str(&status))
        .bind(id)
        .execute(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    async fn list_orders(
        &self,
        exec: &mut dyn DbExecutor,
        customer_id: Uuid,
    ) -> Result<Vec<Order>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let rows = sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT id, customer_id, status, total, note, created_at
            FROM ordering.orders
            WHERE customer_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(customer_id)
        .fetch_all(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_all_orders(&self, exec: &mut dyn DbExecutor) -> Result<Vec<Order>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        let rows = sqlx::query_as::<_, OrderRow>(
            r#"
            SELECT id, customer_id, status, total, note, created_at
            FROM ordering.orders
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_order_note(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
        note: String,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);
        sqlx::query("UPDATE ordering.orders SET note = $1 WHERE id = $2")
            .bind(note)
            .bind(id)
            .execute(&mut *executor.tx)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        Ok(())
    }

    async fn list_customer_recent_orders(
        &self,
        exec: &mut dyn DbExecutor,
        customer_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Order>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);
        let rows = sqlx::query_as::<_, OrderRow>(
            "SELECT id, customer_id, status, total, note, created_at FROM ordering.orders WHERE customer_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(customer_id)
        .bind(limit)
        .fetch_all(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

fn status_to_str(status: &OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "pending",
        OrderStatus::Cancelled => "cancelled",
        OrderStatus::Completed => "completed",
    }
}

// ─────────────────────────────────────────────
// Infrastructure tests (require live Postgres)
// Run with: cargo test -p ordering -- --include-ignored
// ─────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{NewOrderItem, Order};
    use shared::{
        database::UnitOfWork,
        infrastructure::postgres::{PostgresUnitOfWork, SqlxExecutor},
    };
    use std::sync::Arc;

    #[sqlx::test]
    #[ignore = "requires ordering schema in the database"]
    async fn create_and_fetch_order(pool: PgPool) {
        let repo = PostgresOrderRepository::new(pool.clone());
        let uow = PostgresUnitOfWork::new(pool.clone());

        let customer = Some(Uuid::new_v4());
        let items = vec![NewOrderItem {
            variant_id: Uuid::new_v4(),
            quantity: 2,
            unit_price: 500,
        }];

        let (order, order_items) = Order::place(customer, items).unwrap();
        let order_id = order.id;

        // persist
        let repo_arc = Arc::new(repo) as Arc<dyn OrderRepository>;
        let repo_clone = repo_arc.clone();
        let order_clone = order.clone();
        let items_clone = order_items.clone();

        uow.run_atomic(Box::new(move |exec| {
            Box::pin(async move {
                repo_clone
                    .create_order(exec, &order_clone, &items_clone)
                    .await
            })
        }))
        .await
        .unwrap();

        // fetch back
        let tx = pool.begin().await.unwrap();
        let mut executor = SqlxExecutor { tx };
        let fetched = repo_arc
            .get_order_by_id(&mut executor, order_id)
            .await
            .unwrap();

        assert_eq!(fetched.id, order_id);
        assert_eq!(fetched.customer_id, customer);
        assert_eq!(fetched.status, OrderStatus::Pending);
        assert_eq!(fetched.total, 1000);
    }

    #[sqlx::test]
    #[ignore = "requires ordering schema in the database"]
    async fn update_order_status_to_cancelled(pool: PgPool) {
        let repo = Arc::new(PostgresOrderRepository::new(pool.clone())) as Arc<dyn OrderRepository>;
        let uow = PostgresUnitOfWork::new(pool.clone());

        let (order, items) = Order::place(
            Some(Uuid::new_v4()),
            vec![NewOrderItem {
                variant_id: Uuid::new_v4(),
                quantity: 1,
                unit_price: 200,
            }],
        )
        .unwrap();
        let order_id = order.id;

        let r = repo.clone();
        let o = order.clone();
        let i = items.clone();
        uow.run_atomic(Box::new(move |exec| {
            Box::pin(async move { r.create_order(exec, &o, &i).await })
        }))
        .await
        .unwrap();

        let r2 = repo.clone();
        uow.run_atomic(Box::new(move |exec| {
            Box::pin(async move {
                r2.update_order_status(exec, order_id, OrderStatus::Cancelled)
                    .await
            })
        }))
        .await
        .unwrap();

        let tx = pool.begin().await.unwrap();
        let mut executor = SqlxExecutor { tx };
        let fetched = repo.get_order_by_id(&mut executor, order_id).await.unwrap();
        assert_eq!(fetched.status, OrderStatus::Cancelled);
    }
}
