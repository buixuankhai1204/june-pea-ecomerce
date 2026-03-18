use axum::extract::{Path, State, Query};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use shared::AppError;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::model::{NewOrderItem, Order};
use crate::usecase::{
    cancel_order::CancelOrderUsecase, get_order::GetOrderUsecase,
    list_all_orders::ListAllOrdersUsecase, list_orders::ListOrdersUsecase,
    place_order::PlaceOrderUsecase, update_order_status::UpdateOrderStatusUsecase,
    update_order_note::UpdateOrderNoteUsecase,
    list_customer_recent_orders::ListCustomerRecentOrdersUsecase,
};

#[derive(Clone)]
pub struct OrderingUsecase {
    place_order: Arc<PlaceOrderUsecase>,
    cancel_order: Arc<CancelOrderUsecase>,
    get_order: Arc<GetOrderUsecase>,
    list_orders: Arc<ListOrdersUsecase>,
    update_order_status: Arc<UpdateOrderStatusUsecase>,
    list_all_orders: Arc<ListAllOrdersUsecase>,
    update_order_note: Arc<UpdateOrderNoteUsecase>,
    list_recent_orders: Arc<ListCustomerRecentOrdersUsecase>,
}

impl OrderingUsecase {
    pub fn new(
        place_order: Arc<PlaceOrderUsecase>,
        cancel_order: Arc<CancelOrderUsecase>,
        get_order: Arc<GetOrderUsecase>,
        list_orders: Arc<ListOrdersUsecase>,
        update_order_status: Arc<UpdateOrderStatusUsecase>,
        list_all_orders: Arc<ListAllOrdersUsecase>,
        update_order_note: Arc<UpdateOrderNoteUsecase>,
        list_recent_orders: Arc<ListCustomerRecentOrdersUsecase>,
    ) -> Self {
        Self {
            place_order,
            cancel_order,
            get_order,
            list_orders,
            update_order_status,
            list_all_orders,
            update_order_note,
            list_recent_orders,
        }
    }

    pub fn place_order(&self) -> Arc<PlaceOrderUsecase> {
        self.place_order.clone()
    }

    pub fn cancel_order(&self) -> Arc<CancelOrderUsecase> {
        self.cancel_order.clone()
    }

    pub fn get_order(&self) -> Arc<GetOrderUsecase> {
        self.get_order.clone()
    }

    pub fn list_orders(&self) -> Arc<ListOrdersUsecase> {
        self.list_orders.clone()
    }

    pub fn update_order_status(&self) -> Arc<UpdateOrderStatusUsecase> {
        self.update_order_status.clone()
    }

    pub fn list_all_orders(&self) -> Arc<ListAllOrdersUsecase> {
        self.list_all_orders.clone()
    }

    pub fn update_order_note(&self) -> Arc<UpdateOrderNoteUsecase> {
        self.update_order_note.clone()
    }

    pub fn list_recent_orders(&self) -> Arc<ListCustomerRecentOrdersUsecase> {
        self.list_recent_orders.clone()
    }
}

pub fn init() -> Router<OrderingUsecase> {
    Router::new()
        .route(
            "/orders",
            post(place_order_handler).get(list_all_orders_handler),
        )
        .route(
            "/orders/{id}",
            get(get_order_handler).delete(cancel_order_handler),
        )
        .route(
            "/orders/{id}/status",
            axum::routing::patch(update_order_status_handler),
        )
        .route("/orders/{id}/note", axum::routing::patch(update_note_handler))
        .route("/orders/customer/{customer_id}", get(list_orders_handler))
        .route("/orders/customer/{customer_id}/recent", get(list_recent_orders_handler))
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
struct PlaceOrderRequest {
    customer_id: Option<Uuid>,
    items: Vec<NewOrderItem>,
}

#[derive(Debug, Serialize)]
struct PlaceOrderResponse {
    order_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct UpdateOrderStatusRequest {
    status: crate::domain::model::OrderStatus,
}

// --- Handlers ---

async fn place_order_handler(
    State(state): State<OrderingUsecase>,
    Json(body): Json<PlaceOrderRequest>,
) -> Result<Json<PlaceOrderResponse>, AppError> {
    let usecase = state.place_order();
    let order_id = usecase.execute(body.customer_id, body.items).await?;
    Ok(Json(PlaceOrderResponse { order_id }))
}

async fn get_order_handler(
    State(state): State<OrderingUsecase>,
    Path(id): Path<Uuid>,
) -> Result<Json<Order>, AppError> {
    let usecase = state.get_order();
    let order = usecase.execute(id).await?;
    Ok(Json(order))
}

async fn cancel_order_handler(
    State(state): State<OrderingUsecase>,
    Path(id): Path<Uuid>,
) -> Result<Json<bool>, AppError> {
    let usecase = state.cancel_order();
    usecase.execute(id).await?;
    Ok(Json(true))
}

async fn list_orders_handler(
    State(state): State<OrderingUsecase>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<Vec<Order>>, AppError> {
    let usecase = state.list_orders();
    let orders = usecase.execute(customer_id).await?;
    Ok(Json(orders))
}

async fn list_all_orders_handler(
    State(state): State<OrderingUsecase>,
) -> Result<Json<Vec<Order>>, AppError> {
    let usecase = state.list_all_orders();
    let orders = usecase.execute().await?;
    Ok(Json(orders))
}

#[derive(serde::Deserialize)]
pub struct UpdateNoteRequest {
    pub note: String,
}

async fn update_note_handler(
    State(state): State<OrderingUsecase>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let usecase = state.update_order_note();
    usecase.execute(id, payload.note).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[derive(serde::Deserialize)]
pub struct ListRecentOrdersQuery {
    pub limit: Option<i64>,
}

async fn list_recent_orders_handler(
    State(state): State<OrderingUsecase>,
    axum::extract::Path(customer_id): axum::extract::Path<Uuid>,
    Query(query): Query<ListRecentOrdersQuery>,
) -> Result<Json<Vec<Order>>, AppError> {
    let usecase = state.list_recent_orders();
    let limit = query.limit.unwrap_or(5);
    let orders = usecase.execute(customer_id, limit).await?;
    Ok(Json(orders))
}

async fn update_order_status_handler(
    State(state): State<OrderingUsecase>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateOrderStatusRequest>,
) -> Result<Json<bool>, AppError> {
    let usecase = state.update_order_status();
    usecase.execute(id, body.status).await?;
    Ok(Json(true))
}
