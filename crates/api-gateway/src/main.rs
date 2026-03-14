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
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tower_http;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::FmtSubscriber;

mod middleware;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthUsecase>,
    pub catalog_service: Arc<CatalogUsecase>,
    pub inventory_usecase: Arc<InventoryUsecase>,
    pub marketing_usecase: Arc<marketing::routes::MarketingUsecase>,
    pub ordering_usecase: Arc<ordering::routes::OrderingUsecase>,
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
    let marketing_usecases = Arc::new(marketing::routes::MarketingUsecase::new(
        create_coupon,
        validate_coupon,
        list_coupons,
        deactivate_coupon,
        delete_coupon,
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
    let ordering_usecases = Arc::new(ordering::routes::OrderingUsecase::new(
        place_order,
        cancel_order,
        get_order,
        list_orders,
        update_order_status,
        list_all_orders,
    ));

    let marketing_router =
        marketing::routes::init().with_state(marketing_usecases.as_ref().clone());
    let ordering_router = ordering::routes::init().with_state(ordering_usecases.as_ref().clone());
    let catalog_router = catalog::routes::init().with_state(catalog_usecases.as_ref().clone());
    let inventory_router =
        inventory::routes::init().with_state(inventory_usecases.as_ref().clone());

    let state = AppState {
        auth_service: auth_usecases,
        catalog_service: catalog_usecases,
        inventory_usecase: inventory_usecases,
        marketing_usecase: marketing_usecases,
        ordering_usecase: ordering_usecases,
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
        .nest("/api/v1/catalog", catalog_router);

    // Protected routes (auth required)
    let protected_routes = Router::new()
        .nest("/api/v1/inventory", inventory_router)
        .nest("/api/v1/marketing", marketing_router)
        .nest("/api/v1/ordering", ordering_router)
        .layer(axum::middleware::from_fn(middleware::auth::auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .fallback_service(
            tower_http::services::ServeDir::new("dist")
                .fallback(tower_http::services::ServeFile::new("dist/index.html")),
        )
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(
                    "https://eloquent-patience-production.up.railway.app"
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
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
                ]),
        )
        .with_state(state);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr.clone()).await?;
    tracing::info!("🚀 Yame Ecommerce Core started at {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
