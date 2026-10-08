//! Integration tests for the payment side of the database: `PostgresPaymentRepository` and the
//! unit of work against a real Postgres, with every migration applied.
//!
//! `#[sqlx::test]` gives each test its own freshly migrated database on the server named by
//! `DATABASE_URL`, so they need one running (`docker compose up -d postgres`) and are ignored
//! by default:
//!
//! ```text
//! DATABASE_URL=postgres://postgres:postgres@localhost:5432/june_pea \
//!     cargo test -p payment --test integration_postgres -- --ignored
//! ```

mod common;

use chrono::{Duration, Utc};
use common::AlwaysRefunds;
use payment::domain::{PaymentIntent, PaymentProvider, PaymentRepository, PaymentStatus};
use payment::infrastructure::persistence::postgres::PostgresPaymentRepository;
use payment::usecase::refund_payment::RefundPaymentUsecase;
use shared::database::UnitOfWork;
use shared::infrastructure::postgres::PostgresUnitOfWork;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

struct Db {
    pool: PgPool,
    uow: Arc<PostgresUnitOfWork>,
    repo: Arc<PostgresPaymentRepository>,
}

impl Db {
    fn new(pool: PgPool) -> Self {
        Db {
            uow: Arc::new(PostgresUnitOfWork::new(pool.clone())),
            repo: Arc::new(PostgresPaymentRepository::new()),
            pool,
        }
    }

    /// a guest order, the payments table references orders
    async fn seed_order(&self, total: i64) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO ordering.orders (id, customer_id, status, total) VALUES ($1, NULL, 'pending', $2)")
            .bind(id)
            .bind(total)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    async fn create(&self, intent: &PaymentIntent) {
        let (repo, intent) = (self.repo.clone(), intent.clone());
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move { repo.create_intent(exec, &intent).await })
            }))
            .await
            .unwrap();
    }

    async fn set_status(&self, id: Uuid, status: PaymentStatus) -> Result<(), shared::AppError> {
        let repo = self.repo.clone();
        self.uow
            .run_atomic(Box::new(move |exec| {
                Box::pin(async move {
                    repo.update_status(exec, id, status, None, None, None, None)
                        .await
                })
            }))
            .await
    }

    async fn find_by_order(&self, order_id: Uuid) -> Option<PaymentIntent> {
        let (repo, holder) = (self.repo.clone(), Arc::new(Mutex::new(None)));
        let held = holder.clone();
        self.uow
            .run_read_only(Box::new(move |exec| {
                Box::pin(async move {
                    *held.lock().await = repo.find_by_order_id(exec, order_id).await?;
                    Ok(())
                })
            }))
            .await
            .unwrap();
        let found = holder.lock().await.take();
        found
    }

    async fn find_by_txn_ref(&self, txn_ref: &'static str) -> Option<PaymentIntent> {
        let (repo, holder) = (self.repo.clone(), Arc::new(Mutex::new(None)));
        let held = holder.clone();
        self.uow
            .run_read_only(Box::new(move |exec| {
                Box::pin(async move {
                    *held.lock().await = repo.find_by_txn_ref(exec, txn_ref).await?;
                    Ok(())
                })
            }))
            .await
            .unwrap();
        let found = holder.lock().await.take();
        found
    }

    async fn status_in_db(&self, order_id: Uuid) -> String {
        sqlx::query_scalar("SELECT status FROM payment.payments WHERE order_id = $1")
            .bind(order_id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

fn intent(order_id: Uuid, txn_ref: &str) -> PaymentIntent {
    PaymentIntent::new(
        order_id,
        150_000,
        "https://pay.example/checkout".into(),
        txn_ref.into(),
        PaymentProvider::VnPay,
        15,
        Some("203.0.113.9".into()),
    )
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn saves_a_payment_and_finds_it_by_order_and_by_txn_ref(pool: PgPool) {
    let db = Db::new(pool);
    let order_id = db.seed_order(150_000).await;
    let saved = intent(order_id, "TXNREF0001");

    db.create(&saved).await;

    let by_order = db.find_by_order(order_id).await.unwrap();
    assert_eq!(by_order.id, saved.id);
    assert_eq!(by_order.txn_ref, "TXNREF0001");
    assert_eq!(by_order.amount, 150_000);
    assert_eq!(by_order.status, PaymentStatus::Pending);
    assert_eq!(by_order.provider, PaymentProvider::VnPay);
    assert_eq!(by_order.payment_url, "https://pay.example/checkout");
    assert_eq!(by_order.customer_ip.as_deref(), Some("203.0.113.9"));
    // Postgres keeps microseconds, so compare at the precision it stores
    assert_eq!(by_order.expires_at.timestamp_micros(), saved.expires_at.timestamp_micros());
    assert_eq!(db.find_by_txn_ref("TXNREF0001").await.unwrap().id, saved.id);
    assert!(db.find_by_txn_ref("NOSUCHREF").await.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn the_newest_payment_of_an_order_is_the_one_found(pool: PgPool) {
    let db = Db::new(pool);
    let order_id = db.seed_order(150_000).await;
    let mut old = intent(order_id, "TXNREF_OLD");
    old.created_at = Utc::now() - Duration::minutes(30);
    db.create(&old).await;
    db.create(&intent(order_id, "TXNREF_NEW")).await;

    let found = db.find_by_order(order_id).await.unwrap();

    assert_eq!(found.txn_ref, "TXNREF_NEW");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn a_txn_ref_can_only_be_used_once(pool: PgPool) {
    let db = Db::new(pool);
    let order_id = db.seed_order(150_000).await;
    db.create(&intent(order_id, "TXNREF0001")).await;

    let (repo, again) = (db.repo.clone(), intent(order_id, "TXNREF0001"));
    let result = db
        .uow
        .run_atomic(Box::new(move |exec| {
            Box::pin(async move { repo.create_intent(exec, &again).await })
        }))
        .await;

    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn every_payment_status_can_be_stored(pool: PgPool) {
    let db = Db::new(pool);
    for (i, status) in [
        PaymentStatus::Paid,
        PaymentStatus::Failed,
        PaymentStatus::Expired,
        PaymentStatus::Refunded,
    ]
    .into_iter()
    .enumerate()
    {
        let order_id = db.seed_order(150_000).await;
        let saved = intent(order_id, &format!("TXNREF{i}"));
        db.create(&saved).await;

        db.set_status(saved.id, status.clone()).await.unwrap();

        assert_eq!(db.status_in_db(order_id).await, status.as_str());
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn refund_use_case_marks_a_paid_payment_refunded(pool: PgPool) {
    let db = Db::new(pool);
    let order_id = db.seed_order(150_000).await;
    let saved = intent(order_id, "TXNREF0001");
    db.create(&saved).await;
    db.set_status(saved.id, PaymentStatus::Paid).await.unwrap();
    let refund = RefundPaymentUsecase::new(db.repo.clone(), db.uow.clone(), Arc::new(AlwaysRefunds));

    refund.execute(order_id, "admin-1").await.unwrap();

    assert_eq!(db.status_in_db(order_id).await, "refunded");
}
