use crate::domain::{PaymentIntent, PaymentRepository, PaymentStatus};
use crate::usecase::dto::PaymentIntentView;
use qrcode::render::svg;
use qrcode::QrCode;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct GetPaymentStatusUsecase {
    repo: Arc<dyn PaymentRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl GetPaymentStatusUsecase {
    pub fn new(repo: Arc<dyn PaymentRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, order_id: Uuid) -> Result<PaymentIntentView, AppError> {
        let holder = Arc::new(Mutex::new(None));
        let repo = self.repo.clone();
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

        let intent = holder.lock().await.take().ok_or_else(|| {
            AppError::NotFound(format!("No payment intent for order {}", order_id))
        })?;

        Self::intent_to_view(intent)
    }

    fn intent_to_view(intent: PaymentIntent) -> Result<PaymentIntentView, AppError> {
        let qr_svg = if intent.status == PaymentStatus::Pending {
            Some(
                QrCode::new(intent.payment_url.as_bytes())
                    .map_err(|_| AppError::InternalServerError)?
                    .render::<svg::Color>()
                    .min_dimensions(256, 256)
                    .dark_color(svg::Color("#F8FAFC"))
                    .light_color(svg::Color("#0F172A"))
                    .build(),
            )
        } else {
            None
        };
        Ok(PaymentIntentView::from_intent(intent, qr_svg))
    }
}
