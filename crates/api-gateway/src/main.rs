use axum::http::HeaderValue;
use axum::Router;
use catalog::infrastructure::cache::redis::RedisCatalogCache;
use catalog::infrastructure::persistence::postgres::PostgresCatalogRepository;
use catalog::routes::CatalogUsecase;
use dotenv::dotenv;
use identify::infrastructure::persistence::postgres::PostgresUserRepository;
use identify::routes::{init, IdentityState};
use identify::usecase::auth::AuthUsecase;
use inventory::routes::InventoryUsecase;
use payment::config::PaymentConfig;
use payment::infrastructure::persistence::postgres::PostgresPaymentRepository;
use payment::infrastructure::ordering::OrderingAdapter;
use payment::infrastructure::vnpay::{VnPayClient, VnPayGateway};
use payment::routes::PaymentUsecase as PaymentRouter;
use payment::usecase::{
    create_vnpay_qr::CreateVnPayQrUsecase, get_payment_status::GetPaymentStatusUsecase,
    handle_vnpay_ipn::HandleVnPayIpnUsecase,
};
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tower_http;
use tower_http::cors::AllowOrigin;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::FmtSubscriber;
use analytics::routes::AnalyticsUsecase;

mod middleware;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthUsecase>,
    pub catalog_service: Arc<CatalogUsecase>,
    pub inventory_usecase: Arc<InventoryUsecase>,
    pub marketing_usecase: Arc<marketing::routes::MarketingUsecase>,
    pub ordering_usecase: Arc<ordering::routes::OrderingUsecase>,
    pub analytics_usecase: Arc<AnalyticsUsecase>,
    pub user_repo: Arc<PostgresUserRepository>,
}

impl IdentityState for AppState {
    fn auth_service(&self) -> Arc<AuthUsecase> {
        self.auth_service.clone()
    }
    fn get_me_usecase(&self) -> Arc<identify::usecase::get_me::GetMeUsecase> {
        Arc::new(identify::usecase::get_me::GetMeUsecase::new(
            self.user_repo.clone(),
        ))
    }
    fn update_profile_usecase(
        &self,
    ) -> Arc<identify::usecase::update_profile::UpdateProfileUsecase> {
        Arc::new(
            identify::usecase::update_profile::UpdateProfileUsecase::new(self.user_repo.clone()),
        )
    }
    fn list_users_usecase(&self) -> Arc<identify::usecase::list_users::ListUsersUsecase> {
        Arc::new(identify::usecase::list_users::ListUsersUsecase::new(
            self.user_repo.clone(),
        ))
    }
    fn list_memberships_usecase(&self) -> Arc<identify::usecase::list_memberships::ListMembershipsUsecase> {
        Arc::new(identify::usecase::list_memberships::ListMembershipsUsecase::new(
            self.user_repo.clone(),
        ))
    }
    fn get_membership_summary_usecase(&self) -> Arc<identify::usecase::get_membership_summary::GetMembershipSummaryUsecase> {
        Arc::new(identify::usecase::get_membership_summary::GetMembershipSummaryUsecase::new(
            self.user_repo.clone(),
        ))
    }
    fn change_password_usecase(&self) -> Arc<identify::usecase::change_password::ChangePasswordUsecase> {
        Arc::new(identify::usecase::change_password::ChangePasswordUsecase::new(
            self.user_repo.clone(),
        ))
    }
    fn delete_user_usecase(&self) -> Arc<identify::usecase::delete_user::DeleteUserUsecase> {
        Arc::new(identify::usecase::delete_user::DeleteUserUsecase::new(
            self.user_repo.clone(),
        ))
    }
    fn list_staff_usecase(&self) -> Arc<identify::usecase::list_staff::ListStaffUsecase> {
        Arc::new(identify::usecase::list_staff::ListStaffUsecase::new(
            self.user_repo.clone(),
        ))
    }
    fn create_staff_usecase(&self) -> Arc<identify::usecase::create_staff::CreateStaffUsecase> {
        Arc::new(identify::usecase::create_staff::CreateStaffUsecase::new(
            self.user_repo.clone(),
        ))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    subscriber.init();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    print!("Connecting to database at {}... ", db_url);
    let pool = PgPool::connect(&db_url).await?;
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://".to_string());
    print!("Connecting to Redis at {}... ", redis_url);

    let user_repo = Arc::new(PostgresUserRepository::new(Arc::new(pool.clone())));
    let catalog_repo = Arc::new(PostgresCatalogRepository::new(Arc::new(pool.clone())));
    let auth_usecases = Arc::new(AuthUsecase::new(user_repo.clone()));
    let redis = Arc::new(RedisCatalogCache::new(&redis_url).await?);
    let postgrese_unit_of_work = Arc::new(
        shared::infrastructure::postgres::PostgresUnitOfWork::new(pool.clone()),
    );

    // Inventory
    let inventory_usecases = Arc::new(inventory::routes::InventoryUsecase::new(
        Arc::new(inventory::infrastructure::persistence::postgres::PostgresInventoryRepository),
        postgrese_unit_of_work.clone(),
    ));

    let catalog_usecases = Arc::new(CatalogUsecase::new(catalog_repo, redis));

    // Marketing
    let marketing_repo =
        Arc::new(marketing::infrastructure::postgres::PostgresCouponRepository::new(pool.clone()));
    let create_coupon = Arc::new(marketing::usecase::create_coupon::CreateCouponUsecase::new(
        marketing_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let validate_coupon = Arc::new(
        marketing::usecase::validate_coupon::ValidateCouponUsecase::new(
            marketing_repo.clone(),
            postgrese_unit_of_work.clone(),
        ),
    );
    let list_coupons = Arc::new(marketing::usecase::list_coupons::ListCouponsUsecase::new(
        marketing_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let deactivate_coupon = Arc::new(
        marketing::usecase::deactivate_coupon::DeactivateCouponUsecase::new(
            marketing_repo.clone(),
            postgrese_unit_of_work.clone(),
        ),
    );
    let delete_coupon = Arc::new(marketing::usecase::delete_coupon::DeleteCouponUsecase::new(
        marketing_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let apply_category_discount = Arc::new(marketing::usecase::apply_category_discount::ApplyCategoryDiscountUsecase::new(
        marketing_repo.clone(),
    ));
    let marketing_usecases = Arc::new(marketing::routes::MarketingUsecase::new(
        create_coupon,
        validate_coupon,
        list_coupons,
        deactivate_coupon,
        delete_coupon,
        apply_category_discount,
    ));

    // Ordering
    let ordering_repo = Arc::new(
        ordering::infrastructure::persistence::postgres::PostgresOrderRepository::new(pool.clone()),
    );
    let place_order = Arc::new(ordering::usecase::place_order::PlaceOrderUsecase::new(
        ordering_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let cancel_order = Arc::new(ordering::usecase::cancel_order::CancelOrderUsecase::new(
        ordering_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let get_order = Arc::new(ordering::usecase::get_order::GetOrderUsecase::new(
        ordering_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let list_orders = Arc::new(ordering::usecase::list_orders::ListOrdersUsecase::new(
        ordering_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let list_all_orders = Arc::new(
        ordering::usecase::list_all_orders::ListAllOrdersUsecase::new(
            ordering_repo.clone(),
            postgrese_unit_of_work.clone(),
        ),
    );
    let update_order_status = Arc::new(
        ordering::usecase::update_order_status::UpdateOrderStatusUsecase::new(
            ordering_repo.clone(),
            postgrese_unit_of_work.clone(),
        ),
    );
    let update_order_note = Arc::new(ordering::usecase::update_order_note::UpdateOrderNoteUsecase::new(
        ordering_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let list_recent_orders = Arc::new(ordering::usecase::list_customer_recent_orders::ListCustomerRecentOrdersUsecase::new(
        ordering_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let ordering_usecases = Arc::new(ordering::routes::OrderingUsecase::new(
        place_order,
        cancel_order,
        get_order.clone(),
        list_orders,
        update_order_status.clone(),
        list_all_orders,
        update_order_note,
        list_recent_orders,
    ));

    // Analytics
    let analytics_usecases = Arc::new(AnalyticsUsecase::new(Arc::new(pool.clone())));

    // Payment
    let payment_config = PaymentConfig::from_env()?;
    let payment_repo: Arc<dyn payment::domain::PaymentRepository> =
        Arc::new(PostgresPaymentRepository::new());
    let vn_pay_client = Arc::new(VnPayClient::new(payment_config.clone()));
    let order_lookup = Arc::new(OrderingAdapter::new(
        get_order.clone(),
        update_order_status.clone(),
    ));
    let create_vnpay_qr = Arc::new(CreateVnPayQrUsecase::new(
        payment_repo.clone(),
        postgrese_unit_of_work.clone(),
        order_lookup.clone(),
        vn_pay_client.clone(),
        payment_config.clone(),
    ));
    let get_payment_status = Arc::new(GetPaymentStatusUsecase::new(
        payment_repo.clone(),
        postgrese_unit_of_work.clone(),
    ));
    let handle_vnpay_ipn = Arc::new(HandleVnPayIpnUsecase::new(
        payment_repo.clone(),
        postgrese_unit_of_work.clone(),
        vn_pay_client.clone(),
        order_lookup.clone(),
    ));
    let vnpay_gateway = Arc::new(VnPayGateway::new(&payment_config)?);
    let refund_payment = Arc::new(payment::usecase::refund_payment::RefundPaymentUsecase::new(
        payment_repo.clone(),
        postgrese_unit_of_work.clone(),
        vnpay_gateway,
    ));
    let payment_usecases = Arc::new(PaymentRouter::new(
        create_vnpay_qr,
        get_payment_status,
        handle_vnpay_ipn,
        refund_payment,
    ));

    let marketing_router =
        marketing::routes::init().with_state(marketing_usecases.as_ref().clone());
    let marketing_public_router =
        marketing::routes::init_public().with_state(marketing_usecases.as_ref().clone());
    let ordering_router = ordering::routes::init().with_state(ordering_usecases.as_ref().clone());
    let catalog_router = catalog::routes::init().with_state(catalog_usecases.as_ref().clone());
    let inventory_router =
        inventory::routes::init().with_state(inventory_usecases.as_ref().clone());
    let payment_router = payment::routes::init().with_state(payment_usecases.as_ref().clone());
    let payment_ipn_router =
        payment::routes::init_ipn().with_state(payment_usecases.as_ref().clone());
    let analytics_router = analytics::routes::init().with_state(analytics_usecases.as_ref().clone());
    let identity_router = identify::routes::init::<AppState>();
    let cors_layer = tower_http::cors::CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ]);

    let state = AppState {
        auth_service: auth_usecases,
        catalog_service: catalog_usecases,
        inventory_usecase: inventory_usecases,
        marketing_usecase: marketing_usecases,
        ordering_usecase: ordering_usecases,
        analytics_usecase: analytics_usecases,
        user_repo: user_repo,
    };

    tracing::info!("Running database migrations...");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    // Public routes (no auth required)
    let public_routes = Router::new()
        .nest("/api/v1/auth", init())
        .nest("/api/v1/catalog", catalog_router)
        .nest("/api/v1/payment", payment_ipn_router)
        .nest("/api/v1/marketing", marketing_public_router);

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .nest("/api/v1/inventory", inventory_router)
        .nest("/api/v1/marketing", marketing_router)
        .nest("/api/v1/ordering", ordering_router)
        .nest("/api/v1/payment", payment_router)
        .nest("/api/v1/analytics", analytics_router)
        .nest("/api/v1/identity", identity_router)
        .layer(axum::middleware::from_fn(middleware::auth::auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback_service(
            tower_http::services::ServeDir::new("dist")
                .fallback(tower_http::services::ServeFile::new("dist/index.html")),
        )
        .layer(cors_layer)
        .nest_service("/uploads", tower_http::services::ServeDir::new("uploads"))
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await?;
    tracing::info!("🚀 Yame Ecommerce Core started at {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
