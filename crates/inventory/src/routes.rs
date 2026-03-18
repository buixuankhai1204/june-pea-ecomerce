use crate::domain::model::{Stock, Supplier};
use crate::domain::repository::InventoryRepository;
use crate::usecase::{
    decrease_stock::DecreaseStockUsecase, get_stock::GetStockUsecase,
    increase_stock::IncreaseStockUsecase, list_all_stocks::ListAllStocksUsecase,
    update_stock::UpdateStockUsecase,
    list_suppliers::ListSuppliersUsecase,
    create_supplier::CreateSupplierUsecase,
    delete_supplier::DeleteSupplierUsecase,
    check_low_stock_alerts::CheckLowStockAlertsUsecase,
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
    list_suppliers_usecase: Arc<ListSuppliersUsecase>,
    create_supplier_usecase: Arc<CreateSupplierUsecase>,
    delete_supplier_usecase: Arc<DeleteSupplierUsecase>,
    check_low_stock_alerts_usecase: Arc<CheckLowStockAlertsUsecase>,
}

impl InventoryUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self {
            decrease_stock_usecase: Arc::new(DecreaseStockUsecase::new(repo.clone(), uow.clone())),
            increase_stock_usecase: Arc::new(IncreaseStockUsecase::new(repo.clone(), uow.clone())),
            get_stock_usecase: Arc::new(GetStockUsecase::new(repo.clone(), uow.clone())),
            update_stock_usecase: Arc::new(UpdateStockUsecase::new(repo.clone(), uow.clone())),
            list_all_stocks_usecase: Arc::new(ListAllStocksUsecase::new(repo.clone(), uow.clone())),
            list_suppliers_usecase: Arc::new(ListSuppliersUsecase::new(repo.clone(), uow.clone())),
            create_supplier_usecase: Arc::new(CreateSupplierUsecase::new(repo.clone(), uow.clone())),
            delete_supplier_usecase: Arc::new(DeleteSupplierUsecase::new(repo.clone(), uow.clone())),
            check_low_stock_alerts_usecase: Arc::new(CheckLowStockAlertsUsecase::new(repo.clone(), uow.clone())),
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

    pub fn list_suppliers_usecase(&self) -> Arc<ListSuppliersUsecase> {
        self.list_suppliers_usecase.clone()
    }

    pub fn create_supplier_usecase(&self) -> Arc<CreateSupplierUsecase> {
        self.create_supplier_usecase.clone()
    }

    pub fn delete_supplier_usecase(&self) -> Arc<DeleteSupplierUsecase> {
        self.delete_supplier_usecase.clone()
    }

    pub fn check_low_stock_alerts_usecase(&self) -> Arc<CheckLowStockAlertsUsecase> {
        self.check_low_stock_alerts_usecase.clone()
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

#[derive(Debug, Deserialize)]
pub struct CreateSupplierRequest {
    pub name: String,
    pub contact: String,
    pub location: String,
}

pub fn init() -> Router<InventoryUsecase> {
    Router::new()
        .route("/stock/{id}", axum::routing::get(get_stock_handler))
        .route("/decrease-stock", post(decrease_stock_handler))
        .route("/increase-stock", post(increase_stock_handler))
        .route("/update-stock", post(update_stock_handler))
        .route("/list-all", axum::routing::get(list_all_stocks_handler))
        .route("/suppliers", axum::routing::get(list_suppliers_handler))
        .route("/suppliers", post(create_supplier_handler))
        .route("/suppliers/{id}", axum::routing::delete(delete_supplier_handler))
        .route("/stock/low-alerts", axum::routing::get(check_low_stock_handler))
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

async fn list_suppliers_handler(
    State(state): State<InventoryUsecase>,
) -> Result<Json<Vec<Supplier>>, AppError> {
    let usecase = state.list_suppliers_usecase();
    let suppliers = usecase.execute().await?;
    Ok(Json(suppliers))
}

async fn create_supplier_handler(
    State(state): State<InventoryUsecase>,
    Json(body): Json<CreateSupplierRequest>,
) -> Result<Json<bool>, AppError> {
    let usecase = state.create_supplier_usecase();
    usecase.execute(body.name, body.contact, body.location).await?;
    Ok(Json(true))
}

async fn delete_supplier_handler(
    State(state): State<InventoryUsecase>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<bool>, AppError> {
    let usecase = state.delete_supplier_usecase();
    usecase.execute(id).await?;
    Ok(Json(true))
}

async fn check_low_stock_handler(
    State(state): State<InventoryUsecase>,
    axum::extract::Query(query): axum::extract::Query<CheckLowStockQuery>,
) -> Result<Json<Vec<Stock>>, AppError> {
    let usecase = state.check_low_stock_alerts_usecase();
    let threshold = query.threshold.unwrap_or(10);
    let stocks = usecase.execute(threshold).await?;
    Ok(Json(stocks))
}

#[derive(Debug, Deserialize)]
pub struct CheckLowStockQuery {
    pub threshold: Option<i32>,
}
