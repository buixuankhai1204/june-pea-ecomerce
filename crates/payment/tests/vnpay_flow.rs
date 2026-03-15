use std::collections::BTreeMap;
use std::sync::Arc;

use hmac::{Hmac, Mac};
use ordering::infrastructure::persistence::postgres::PostgresOrderRepository;
use ordering::usecase::get_order::GetOrderUsecase;
use ordering::usecase::update_order_status::UpdateOrderStatusUsecase;
use payment::config::PaymentConfig;
use payment::domain::{PaymentRepository, PaymentStatus};
use payment::infrastructure::persistence::postgres::PostgresPaymentRepository;
use payment::infrastructure::vnpay::VnPayClient;
use payment::usecase::create_vnpay_qr::CreateVnPayQrUsecase;
use payment::usecase::handle_vnpay_ipn::HandleVnPayIpnUsecase;
use sha2::Sha512;
use shared::infrastructure::postgres::PostgresUnitOfWork;
use sqlx::PgPool;
use urlencoding::encode;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn create_vnpay_qr_persists_payment(pool: PgPool) {
    let customer_id = seed_user(&pool).await;
    let order_id = seed_order(&pool, customer_id, 150_000).await;

    let uow = Arc::new(PostgresUnitOfWork::new(pool.clone()));
    let order_repo: Arc<dyn ordering::domain::repository::OrderRepository> =
        Arc::new(PostgresOrderRepository::new(pool.clone()));
    let payment_repo: Arc<dyn PaymentRepository> = Arc::new(PostgresPaymentRepository::new());
    let get_order = Arc::new(GetOrderUsecase::new(order_repo.clone(), uow.clone()));
    let config = test_config();
    let vn_pay_client = Arc::new(VnPayClient::new(config.clone()));
    let create_usecase = CreateVnPayQrUsecase::new(
        payment_repo.clone(),
        uow.clone(),
        get_order,
        vn_pay_client,
        config.clone(),
    );

    let intent = create_usecase
        .execute(order_id, Some("127.0.0.1".into()))
        .await
        .expect("intent created");
    assert_eq!(intent.status, PaymentStatus::Pending);
    assert!(intent.qr_svg.is_some());

    let (status, txn_ref): (String, String) =
        sqlx::query_as("SELECT status, txn_ref FROM payment.payments WHERE order_id = $1")
            .bind(order_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(status, "pending");
    assert_eq!(txn_ref, intent.txn_ref);
}

#[sqlx::test(migrations = "../../migrations")]
async fn handle_vnpay_ipn_marks_order_paid(pool: PgPool) {
    let customer_id = seed_user(&pool).await;
    let order_id = seed_order(&pool, customer_id, 220_000).await;

    let uow = Arc::new(PostgresUnitOfWork::new(pool.clone()));
    let order_repo: Arc<dyn ordering::domain::repository::OrderRepository> =
        Arc::new(PostgresOrderRepository::new(pool.clone()));
    let payment_repo: Arc<dyn PaymentRepository> = Arc::new(PostgresPaymentRepository::new());
    let get_order = Arc::new(GetOrderUsecase::new(order_repo.clone(), uow.clone()));
    let update_order_status = Arc::new(UpdateOrderStatusUsecase::new(
        order_repo.clone(),
        uow.clone(),
    ));

    let config = test_config();
    let vn_pay_client = Arc::new(VnPayClient::new(config.clone()));
    let create_usecase = CreateVnPayQrUsecase::new(
        payment_repo.clone(),
        uow.clone(),
        get_order,
        vn_pay_client.clone(),
        config.clone(),
    );
    let handle_ipn = HandleVnPayIpnUsecase::new(
        payment_repo,
        uow.clone(),
        vn_pay_client,
        update_order_status,
    );

    let intent = create_usecase
        .execute(order_id, Some("127.0.0.1".into()))
        .await
        .expect("intent created");

    let mut params = BTreeMap::new();
    params.insert("vnp_TxnRef".into(), intent.txn_ref.clone());
    params.insert("vnp_Amount".into(), format!("{}", intent.amount * 100));
    params.insert("vnp_ResponseCode".into(), "00".into());
    params.insert("vnp_TransactionStatus".into(), "00".into());
    params.insert("vnp_OrderInfo".into(), "Test order".into());
    params.insert("vnp_SecureHashType".into(), "HmacSHA512".into());
    let secure_hash = sign_params(&config.hash_secret, &params);
    params.insert("vnp_SecureHash".into(), secure_hash);

    let response = handle_ipn
        .execute(params)
        .await
        .expect("ipn handled successfully");
    assert_eq!(response.rsp_code, "00");

    let status: (String,) =
        sqlx::query_as("SELECT status FROM payment.payments WHERE order_id = $1")
            .bind(order_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status.0, "paid");

    let order_status: (String,) =
        sqlx::query_as("SELECT status FROM ordering.orders WHERE id = $1")
            .bind(order_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(order_status.0, "completed");
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO identify.users (id, email, password_hash, role) VALUES ($1, $2, $3, 'customer')",
    )
    .bind(user_id)
    .bind(format!("user+{}@example.com", user_id))
    .bind("hashed")
    .execute(pool)
    .await
    .unwrap();
    user_id
}

async fn seed_order(pool: &PgPool, customer_id: Uuid, total: i64) -> Uuid {
    let order_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO ordering.orders (id, customer_id, status, total) VALUES ($1, $2, 'pending', $3)",
    )
    .bind(order_id)
    .bind(customer_id)
    .bind(total)
    .execute(pool)
    .await
    .unwrap();
    order_id
}

fn test_config() -> PaymentConfig {
    PaymentConfig {
        tmn_code: "LINSR88X".into(),
        hash_secret: "WF46RHBU0XEMXMVE1ZSJSHWFZDAK53S".into(),
        payment_url: "https://sandbox.vnpayment.vn/paymentv2/vpcpay.html".into(),
        return_url: "https://localhost:8080/payment/vnpay/return".into(),
        ipn_url: "http://localhost:3000/api/v1/payment/vnpay/ipn".into(),
        default_locale: "vn".into(),
        order_type: "fashion".into(),
        qr_expiry_minutes: 15,
    }
}

fn sign_params(secret: &str, params: &BTreeMap<String, String>) -> String {
    let mut filtered = params.clone();
    filtered.remove("vnp_SecureHash");
    filtered.remove("vnp_SecureHashType");
    let query = filtered
        .iter()
        .map(|(k, v)| format!("{}={}", k, encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let mut mac = Hmac::<Sha512>::new_from_slice(secret.as_bytes()).expect("valid key");
    mac.update(query.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
