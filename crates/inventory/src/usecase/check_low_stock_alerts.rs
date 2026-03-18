use crate::domain::model::Stock;
use crate::domain::repository::InventoryRepository;
use shared::{database::UnitOfWork, error::AppError};
use std::sync::Arc;

pub struct CheckLowStockAlertsUsecase {
    repo: Arc<dyn InventoryRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl CheckLowStockAlertsUsecase {
    pub fn new(repo: Arc<dyn InventoryRepository>, uow: Arc<dyn UnitOfWork>) -> Self {
        Self { repo, uow }
    }

    pub async fn execute(&self, threshold: i32) -> Result<Vec<Stock>, AppError> {
        let repo = self.repo.clone();
        let fetched_stocks = Arc::new(tokio::sync::Mutex::new(None));
        let fetched_stocks_clone = fetched_stocks.clone();

        self.uow.run_atomic(Box::new(move |exec| {
            Box::pin(async move {
                let stocks = repo.list_low_stock(exec, threshold).await?;
                *fetched_stocks_clone.lock().await = Some(stocks);
                Ok(())
            })
        })).await?;

        let stocks = fetched_stocks.lock().await.take().unwrap();
        Ok(stocks)
    }
}
