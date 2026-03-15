use crate::usecase::{
    create_vnpay_qr::CreateVnPayQrUsecase,
    dto::PaymentIntentView,
    get_payment_status::GetPaymentStatusUsecase,
    handle_vnpay_ipn::{HandleVnPayIpnUsecase, IpnResponseBody},
};
use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_qs::from_str;
use shared::AppError;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct PaymentUsecase {
    create_vnpay_qr: Arc<CreateVnPayQrUsecase>,
    get_payment_status: Arc<GetPaymentStatusUsecase>,
    handle_vnpay_ipn: Arc<HandleVnPayIpnUsecase>,
}

impl PaymentUsecase {
    pub fn new(
        create_vnpay_qr: Arc<CreateVnPayQrUsecase>,
        get_payment_status: Arc<GetPaymentStatusUsecase>,
        handle_vnpay_ipn: Arc<HandleVnPayIpnUsecase>,
    ) -> Self {
        Self {
            create_vnpay_qr,
            get_payment_status,
            handle_vnpay_ipn,
        }
    }

    pub fn create_vnpay_qr(&self) -> Arc<CreateVnPayQrUsecase> {
        self.create_vnpay_qr.clone()
    }

    pub fn get_payment_status(&self) -> Arc<GetPaymentStatusUsecase> {
        self.get_payment_status.clone()
    }

    pub fn handle_vnpay_ipn(&self) -> Arc<HandleVnPayIpnUsecase> {
        self.handle_vnpay_ipn.clone()
    }
}

pub fn init() -> Router<PaymentUsecase> {
    Router::new()
        .route("/vnpay/qr", post(create_vnpay_qr_handler))
        .route("/orders/{order_id}", get(get_payment_status_handler))
}

pub fn init_ipn() -> Router<PaymentUsecase> {
    Router::new().route("/vnpay/ipn", get(vnpay_ipn_handler).post(vnpay_ipn_handler))
}

#[derive(Debug, Deserialize)]
struct CreateVnPayQrRequest {
    order_id: Uuid,
}

async fn create_vnpay_qr_handler(
    State(state): State<PaymentUsecase>,
    Json(payload): Json<CreateVnPayQrRequest>,
) -> Result<Json<PaymentIntentView>, AppError> {
    let usecase = state.create_vnpay_qr();
    let view = usecase.execute(payload.order_id, None).await?;
    Ok(Json(view))
}

async fn get_payment_status_handler(
    State(state): State<PaymentUsecase>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<PaymentIntentView>, AppError> {
    let usecase = state.get_payment_status();
    let view = usecase.execute(order_id).await?;
    Ok(Json(view))
}

async fn vnpay_ipn_handler(
    State(state): State<PaymentUsecase>,
    RawQuery(query): RawQuery,
) -> (StatusCode, Json<IpnResponseBody>) {
    let params = match query {
        Some(q) => match from_str::<BTreeMap<String, String>>(&q) {
            Ok(map) => map,
            Err(_) => {
                return (
                    StatusCode::OK,
                    Json(IpnResponseBody::error("99", "Invalid query")),
                )
            }
        },
        None => {
            return (
                StatusCode::OK,
                Json(IpnResponseBody::error("99", "Missing query")),
            )
        }
    };

    match state.handle_vnpay_ipn().execute(params).await {
        Ok(body) => (StatusCode::OK, Json(body)),
        Err(err) => {
            error!(error = ?err, "VNPay IPN handler failed");
            (
                StatusCode::OK,
                Json(IpnResponseBody::error("99", "Internal error")),
            )
        }
    }
}
