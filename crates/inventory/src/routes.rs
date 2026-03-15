use crate::domain::model::Stock;
use crate::domain::repository::InventoryRepository;
use crate::usecase::{
    decrease_stock::DecreaseStockUsecase, get_stock::GetStockUsecase,
    increase_stock::IncreaseStockUsecase, list_all_stocks::ListAllStocksUsecase,
    update_stock::UpdateStockUsecase,
};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use shared::database::UnitOfWork;
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct InventoryUsecase {
    decrease_stock_usecase: Arc<DecreaseStockUsecase>,
    increase_stock_usecase: Arc<IncreaseStockUsecase>,
    get_stock_usecase: Arc<GetStockUsecase>,
    update_stock_usecase: Arc<UpdateStockUsecase>,
    list_all_stocks_usecase: Arc<ListAllStocksUsecase>,
}

impl InventoryUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self {
            decrease_stock_usecase: Arc::new(DecreaseStockUsecase::new(repo.clone(), uow.clone())),
            increase_stock_usecase: Arc::new(IncreaseStockUsecase::new(repo.clone(), uow.clone())),
            get_stock_usecase: Arc::new(GetStockUsecase::new(repo.clone(), uow.clone())),
            update_stock_usecase: Arc::new(UpdateStockUsecase::new(repo.clone(), uow.clone())),
            list_all_stocks_usecase: Arc::new(ListAllStocksUsecase::new(repo, uow)),
        }
    }

    pub fn decrease_stock_usecase(&self) -> Arc<DecreaseStockUsecase> {
        self.decrease_stock_usecase.clone()
    }

    pub fn increase_stock_usecase(&self) -> Arc<IncreaseStockUsecase> {
        self.increase_stock_usecase.clone()
    }

    pub fn get_stock_usecase(&self) -> Arc<GetStockUsecase> {
        self.get_stock_usecase.clone()
    }

    pub fn update_stock_usecase(&self) -> Arc<UpdateStockUsecase> {
        self.update_stock_usecase.clone()
    }

    pub fn list_all_stocks_usecase(&self) -> Arc<ListAllStocksUsecase> {
        self.list_all_stocks_usecase.clone()
    }
}

#[derive(Debug, Deserialize)]
struct StockRequest {
    variant_id: Uuid,
    amount: i32,
}

#[derive(Debug, Deserialize)]
struct UpdateStockRequest {
    variant_id: Uuid,
    quantity: i32,
}

pub fn init() -> Router<InventoryUsecase> {
    Router::new()
        .route("/stock/{id}", axum::routing::get(get_stock_handler))
        .route("/decrease-stock", post(decrease_stock_handler))
        .route("/increase-stock", post(increase_stock_handler))
        .route("/update-stock", post(update_stock_handler))
        .route("/list-all", axum::routing::get(list_all_stocks_handler))
}

async fn decrease_stock_handler(
    State(state): State<InventoryUsecase>,
    Json(body): Json<StockRequest>,
) -> Result<Json<bool>, AppError> {
    if body.amount <= 0 {
        return Err(AppError::Validation("Amount must be positive".into()));
    }
    let usecase = state.decrease_stock_usecase();
    usecase.execute(body.variant_id, body.amount).await?;
    Ok(Json(true))
}

async fn increase_stock_handler(
    State(state): State<InventoryUsecase>,
    Json(body): Json<StockRequest>,
) -> Result<Json<bool>, AppError> {
    if body.amount <= 0 {
        return Err(AppError::Validation("Amount must be positive".into()));
    }
    let usecase = state.increase_stock_usecase();
    usecase.execute(body.variant_id, body.amount).await?;
    Ok(Json(true))
}

async fn update_stock_handler(
    State(state): State<InventoryUsecase>,
    Json(body): Json<UpdateStockRequest>,
) -> Result<Json<bool>, AppError> {
    let usecase = state.update_stock_usecase();
    usecase.execute(body.variant_id, body.quantity).await?;
    Ok(Json(true))
}

async fn get_stock_handler(
    State(state): State<InventoryUsecase>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<i32>, AppError> {
    let usecase = state.get_stock_usecase();
    let stock = usecase.execute(id).await?;
    Ok(Json(stock))
}

async fn list_all_stocks_handler(
    State(state): State<InventoryUsecase>,
) -> Result<Json<Vec<Stock>>, AppError> {
    let usecase = state.list_all_stocks_usecase();
    let stocks = usecase.execute().await?;
    Ok(Json(stocks))
}
