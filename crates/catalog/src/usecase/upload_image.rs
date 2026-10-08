use crate::domain::cache::CatalogCache;
use crate::domain::catalog_repository::CatalogRepository;
use shared::AppError;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
use image::imageops::FilterType;

pub struct UploadProductImageUsecase {
    repo: Arc<dyn CatalogRepository>,
    cache: Arc<dyn CatalogCache>,
}

impl UploadProductImageUsecase {
    pub fn new(repo: Arc<dyn CatalogRepository>, cache: Arc<dyn CatalogCache>) -> Self {
        Self { repo, cache }
    }

    pub async fn execute(
        &self,
        product_id: Uuid,
        file_name: &str,
        content: Vec<u8>,
    ) -> Result<String, AppError> {
        let extension = Path::new(file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");
        
        let unique_id = Uuid::new_v4();
        // Standardize extension to jpg if it's common, or keep original if it's webp/png
        let final_extension = match extension.to_lowercase().as_str() {
            "png" => "png",
            "webp" => "webp",
            _ => "jpg",
        };

        let base_name = format!("{}_{}", product_id, unique_id);
        
        // Define paths (relative to workspace root)
        let upload_dir = "uploads";
        let standard_dir = format!("{}/standard", upload_dir);
        let mobile_dir = format!("{}/mobile", upload_dir);
        let thumbnail_dir = format!("{}/thumbnail", upload_dir);
        
        // Ensure directories exist
        std::fs::create_dir_all(&standard_dir).map_err(|e| {
            tracing::error!("Failed to create standard directory: {}", e);
            AppError::InternalServerError
        })?;
        std::fs::create_dir_all(&mobile_dir).map_err(|e| {
            tracing::error!("Failed to create mobile directory: {}", e);
            AppError::InternalServerError
        })?;
        std::fs::create_dir_all(&thumbnail_dir).map_err(|e| {
            tracing::error!("Failed to create thumbnail directory: {}", e);
            AppError::InternalServerError
        })?;
        
        // Load image
        let img = image::load_from_memory(&content).map_err(|e| AppError::Validation(format!("Invalid image: {}", e)))?;
        
        // Standard: 1200x1600 (3:4)
        let standard_img = img.resize_to_fill(1200, 1600, FilterType::Lanczos3);
        let standard_rel_path = format!("standard/{}.{}", base_name, final_extension);
        let standard_full_path = format!("{}/{}", upload_dir, standard_rel_path);
        standard_img.save(&standard_full_path).map_err(|e| {
            tracing::error!("Failed to save standard image: {}", e);
            AppError::InternalServerError
        })?;
        
        // Mobile: 600x800
        let mobile_img = img.resize_to_fill(600, 800, FilterType::Lanczos3);
        let mobile_full_path = format!("{}/mobile/{}.{}", upload_dir, base_name, final_extension);
        mobile_img.save(&mobile_full_path).map_err(|e| {
            tracing::error!("Failed to save mobile image: {}", e);
            AppError::InternalServerError
        })?;
        
        // Thumbnail: 200x267
        let thumbnail_img = img.resize_to_fill(200, 267, FilterType::Lanczos3);
        let thumbnail_full_path = format!("{}/thumbnail/{}.{}", upload_dir, base_name, final_extension);
        thumbnail_img.save(&thumbnail_full_path).map_err(|e| {
            tracing::error!("Failed to save thumbnail image: {}", e);
            AppError::InternalServerError
        })?;
        
        // Save to DB (storing the standard path as the main URL)
        let public_url = format!("/uploads/{}", standard_rel_path);
        
        // Find existing image count to set position
        // let existing_images = self.repo.get_product_images(product_id).await?;
        // let position = (existing_images.len() as i32) + 1;
        // let is_primary = existing_images.is_empty();
        
        // self.repo.add_product_image(unique_id, product_id, &public_url, is_primary, position).await?;
        
        // Invalidate cache
        if let Ok(Some(product_with_variants)) = self.repo.get_by_id(product_id).await {
            let _ = self.cache.delete_product(&product_with_variants.product.slug).await;
        }

        Ok(public_url)
    }
}
