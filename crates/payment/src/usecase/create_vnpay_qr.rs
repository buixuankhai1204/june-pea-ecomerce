use crate::config::PaymentConfig;
use crate::domain::{PaymentIntent, PaymentProvider, PaymentRepository, PaymentStatus};
use crate::infrastructure::vnpay::{VnPayClient, VnPayPaymentRequest};
use crate::usecase::dto::PaymentIntentView;
use ordering::domain::model::OrderStatus;
use ordering::usecase::get_order::GetOrderUsecase;
use qrcode::render::svg;
use qrcode::QrCode;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct CreateVnPayQrUsecase {
    repo: Arc<dyn PaymentRepository>,
    uow: Arc<dyn UnitOfWork>,
    get_order: Arc<GetOrderUsecase>,
    vn_pay_client: Arc<VnPayClient>,
    config: PaymentConfig,
}

impl CreateVnPayQrUsecase {
    pub fn new(
        repo: Arc<dyn PaymentRepository>,
        uow: Arc<dyn UnitOfWork>,
        get_order: Arc<GetOrderUsecase>,
        vn_pay_client: Arc<VnPayClient>,
        config: PaymentConfig,
    ) -> Self {
        Self {
            repo,
            uow,
            get_order,
            vn_pay_client,
            config,
        }
    }

    pub async fn execute(
        &self,
        order_id: Uuid,
        client_ip: Option<String>,
    ) -> Result<PaymentIntentView, AppError> {
        let order = self.get_order.execute(order_id).await?;
        if order.status != OrderStatus::Pending {
            return Err(AppError::Conflict("Order is not pending".into()));
        }

        if order.total <= 0 {
            return Err(AppError::Validation("Order total must be positive".into()));
        }

        if let Some(existing) = self.fetch_latest(order_id).await? {
            if existing.status == PaymentStatus::Pending {
                if existing.is_expired() {
                    self.mark_status(existing.id, PaymentStatus::Expired)
                        .await?;
                } else {
                    return Self::intent_to_view(existing);
                }
            } else if existing.status == PaymentStatus::Paid {
                return Self::intent_to_view(existing);
            }
        }

        let txn_ref = VnPayClient::random_txn_ref();
        let expire_at =
            chrono::Utc::now() + chrono::Duration::minutes(self.config.qr_expiry_minutes);
        let payment_request = VnPayPaymentRequest {
            txn_ref: txn_ref.clone(),
            amount: order.total,
            order_info: format!("Thanh toan don hang {}", order.id),
            client_ip: client_ip.clone(),
            expire_at,
        };

        let payment_response = self
            .vn_pay_client
            .build_payment_url(&payment_request)
            .map_err(|_| AppError::InternalServerError)?;

        let intent = PaymentIntent::new(
            order_id,
            order.total,
            payment_response.payment_url.clone(),
            txn_ref,
            PaymentProvider::VnPay,
            self.config.qr_expiry_minutes,
            client_ip,
        );

        let repo = self.repo.clone();
        let intent_clone = intent.clone();
        self.uow
            .run_atomic(Box::new(move |exec| {
                let repo = repo.clone();
                let intent = intent_clone.clone();
                Box::pin(async move { repo.create_intent(exec, &intent).await })
            }))
            .await?;

        Self::intent_to_view(intent)
    }

    async fn fetch_latest(&self, order_id: Uuid) -> Result<Option<PaymentIntent>, AppError> {
        let repo = self.repo.clone();
        let holder = Arc::new(Mutex::new(None));
        let holder_clone = holder.clone();
        self.uow
            .run_read_only(Box::new(move |exec| {
                let repo = repo.clone();
                let holder = holder_clone.clone();
                Box::pin(async move {
                    let payment = repo.find_by_order_id(exec, order_id).await?;
                    *holder.lock().await = payment;
                    Ok(())
                })
            }))
            .await?;

        let result = holder.lock().await.take();
        Ok(result)
    }

    async fn mark_status(&self, payment_id: Uuid, status: PaymentStatus) -> Result<(), AppError> {
        let repo = self.repo.clone();
        self.uow
            .run_atomic(Box::new(move |exec| {
                let repo = repo.clone();
                Box::pin(async move {
                    repo.update_status(exec, payment_id, status, None, None, None, None)
                        .await
                })
            }))
            .await
    }

    fn intent_to_view(intent: PaymentIntent) -> Result<PaymentIntentView, AppError> {
        let qr_svg = if intent.status == PaymentStatus::Pending {
            Some(Self::generate_svg(&intent.payment_url)?)
        } else {
            None
        };
        Ok(PaymentIntentView::from_intent(intent, qr_svg))
    }

    fn generate_svg(url: &str) -> Result<String, AppError> {
        let code = QrCode::new(url.as_bytes()).map_err(|_| AppError::InternalServerError)?;
        let image = code
            .render::<svg::Color>()
            .min_dimensions(256, 256)
            .dark_color(svg::Color("#F8FAFC"))
            .light_color(svg::Color("#0F172A"))
            .build();
        Ok(image)
    }
}
