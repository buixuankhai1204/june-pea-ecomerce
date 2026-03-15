use crate::domain::cache::CatalogCache;
use crate::domain::model::ProductWithVariants;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use shared::AppError;

pub struct RedisCatalogCache {
    connection_manager: ConnectionManager,
    ttl_seconds: u64,
}

impl RedisCatalogCache {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url).map_err(|_e| AppError::InternalServerError)?;

        let manager = ConnectionManager::new(client)
            .await
            .map_err(|_e| AppError::InternalServerError)?;

        Ok(Self {
            connection_manager: manager,
            ttl_seconds: 3600, // 1 hour default TTL
        })
    }
}

#[async_trait]
impl CatalogCache for RedisCatalogCache {
    async fn get_product(&self, slug: &str) -> Result<Option<ProductWithVariants>, AppError> {
        let key = format!("catalog:product:{}", slug);
        let mut conn = self.connection_manager.clone();

        let cached_data: Option<String> = conn
            .get(&key)
            .await
            .map_err(|_e| AppError::InternalServerError)?;

        match cached_data {
            Some(json) => {
                let product =
                    serde_json::from_str(&json).map_err(|_e| AppError::InternalServerError)?;
                Ok(Some(product))
            }
            None => Ok(None),
        }
    }

    async fn set_product(&self, slug: &str, data: &ProductWithVariants) -> Result<(), AppError> {
        let key = format!("catalog:product:{}", slug);
        let mut conn = self.connection_manager.clone();

        let json = serde_json::to_string(data).map_err(|_e| AppError::InternalServerError)?;

        // Use SETEX logic: Set value with expiration
        let _: () = conn
            .set_ex(&key, json, self.ttl_seconds)
            .await
            .map_err(|_e| AppError::InternalServerError)?;

        Ok(())
    }

    async fn delete_product(&self, slug: &str) -> Result<(), AppError> {
        let key = format!("catalog:product:{}", slug);
        let mut conn = self.connection_manager.clone();

        let _: () = conn
            .del(&key)
            .await
            .map_err(|_e| AppError::InternalServerError)?;

        Ok(())
    }
}
