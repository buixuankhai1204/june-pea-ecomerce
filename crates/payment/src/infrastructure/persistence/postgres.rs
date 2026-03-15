use crate::domain::{
    model::{PaymentIntent, PaymentProvider, PaymentStatus},
    repository::PaymentRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use shared::{database::DbExecutor, error::AppError, infrastructure::postgres::SqlxExecutor};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct PaymentRow {
    pub id: Uuid,
    pub order_id: Uuid,
    pub provider: String,
    pub txn_ref: String,
    pub amount: i64,
    pub status: String,
    pub payment_url: String,
    pub response_code: Option<String>,
    pub transaction_status: Option<String>,
    pub transaction_no: Option<String>,
    pub bank_code: Option<String>,
    pub customer_ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

impl From<PaymentRow> for PaymentIntent {
    fn from(row: PaymentRow) -> Self {
        Self {
            id: row.id,
            order_id: row.order_id,
            provider: PaymentProvider::from_str(&row.provider),
            txn_ref: row.txn_ref,
            amount: row.amount,
            status: PaymentStatus::from_str(&row.status),
            payment_url: row.payment_url,
            response_code: row.response_code,
            transaction_status: row.transaction_status,
            transaction_no: row.transaction_no,
            bank_code: row.bank_code,
            customer_ip: row.customer_ip,
            created_at: row.created_at,
            updated_at: row.updated_at,
            expires_at: row.expires_at,
            paid_at: row.paid_at,
        }
    }
}

pub struct PostgresPaymentRepository;

impl PostgresPaymentRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PaymentRepository for PostgresPaymentRepository {
    async fn create_intent(
        &self,
        exec: &mut dyn DbExecutor,
        intent: &PaymentIntent,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);

        sqlx::query(
            r#"
            INSERT INTO payment.payments (
                id, order_id, provider, txn_ref, amount, status, payment_url,
                response_code, transaction_status, transaction_no, bank_code,
                customer_ip, created_at, updated_at, expires_at, paid_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, $9, $10, $11,
                $12, $13, $14, $15, $16
            )
            "#,
        )
        .bind(intent.id)
        .bind(intent.order_id)
        .bind(intent.provider.as_str())
        .bind(&intent.txn_ref)
        .bind(intent.amount)
        .bind(intent.status.as_str())
        .bind(&intent.payment_url)
        .bind(&intent.response_code)
        .bind(&intent.transaction_status)
        .bind(&intent.transaction_no)
        .bind(&intent.bank_code)
        .bind(&intent.customer_ip)
        .bind(intent.created_at)
        .bind(intent.updated_at)
        .bind(intent.expires_at)
        .bind(intent.paid_at)
        .execute(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    async fn update_intent(
        &self,
        exec: &mut dyn DbExecutor,
        intent: &PaymentIntent,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);
        sqlx::query(
            r#"
            UPDATE payment.payments SET
                status = $2,
                response_code = $3,
                transaction_status = $4,
                transaction_no = $5,
                bank_code = $6,
                customer_ip = $7,
                payment_url = $8,
                updated_at = NOW(),
                expires_at = $9,
                paid_at = $10
            WHERE id = $1
            "#,
        )
        .bind(intent.id)
        .bind(intent.status.as_str())
        .bind(&intent.response_code)
        .bind(&intent.transaction_status)
        .bind(&intent.transaction_no)
        .bind(&intent.bank_code)
        .bind(&intent.customer_ip)
        .bind(&intent.payment_url)
        .bind(intent.expires_at)
        .bind(intent.paid_at)
        .execute(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    async fn find_by_order_id(
        &self,
        exec: &mut dyn DbExecutor,
        order_id: Uuid,
    ) -> Result<Option<PaymentIntent>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT * FROM payment.payments
            WHERE order_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(order_id)
        .fetch_optional(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(row.map(Into::into))
    }

    async fn find_by_txn_ref(
        &self,
        exec: &mut dyn DbExecutor,
        txn_ref: &str,
    ) -> Result<Option<PaymentIntent>, AppError> {
        let executor = SqlxExecutor::from_executor(exec);
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT * FROM payment.payments
            WHERE txn_ref = $1
            LIMIT 1
            "#,
        )
        .bind(txn_ref)
        .fetch_optional(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(row.map(Into::into))
    }

    async fn update_status(
        &self,
        exec: &mut dyn DbExecutor,
        id: Uuid,
        status: PaymentStatus,
        response_code: Option<String>,
        transaction_status: Option<String>,
        transaction_no: Option<String>,
        bank_code: Option<String>,
    ) -> Result<(), AppError> {
        let executor = SqlxExecutor::from_executor(exec);
        sqlx::query(
            r#"
            UPDATE payment.payments
            SET status = $2,
                response_code = $3,
                transaction_status = $4,
                transaction_no = $5,
                bank_code = $6,
                updated_at = NOW(),
                paid_at = CASE WHEN $2 = 'paid' THEN NOW() ELSE paid_at END
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status.as_str())
        .bind(response_code)
        .bind(transaction_status)
        .bind(transaction_no)
        .bind(bank_code)
        .execute(&mut *executor.tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }
}
