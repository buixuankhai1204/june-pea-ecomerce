use catalog::{
    infrastructure::persistence::postgres::PostgresCatalogRepository,
    usecase::{
        create_category::CreateCategoryUsecase, create_product::CreateProductUsecase,
        list_categories::ListCategoriesUsecase, update_product::UpdateProductUsecase,
    },
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

struct TestContext {
    create_category: CreateCategoryUsecase,
    create_product: CreateProductUsecase,
    list_categories: ListCategoriesUsecase,
    update_product: UpdateProductUsecase,
}

impl TestContext {
    fn new(pool: PgPool) -> Self {
        let repo = Arc::new(PostgresCatalogRepository::new(Arc::new(pool)));
        Self {
            create_category: CreateCategoryUsecase::new(repo.clone()),
            create_product: CreateProductUsecase::new(repo.clone()),
            list_categories: ListCategoriesUsecase::new(repo.clone()),
            update_product: UpdateProductUsecase::new(repo),
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn e2e_create_category_and_product_works(pool: PgPool) {
    let ctx = TestContext::new(pool.clone());

    // 1. Create Category
    let category_id = ctx
        .create_category
        .execute(
            "Electronics".to_string(),
            Some(format!("electronics-{}", Uuid::new_v4())),
            None,
        )
        .await
        .unwrap();

    // 2. Create Product in that category
    let product_id = ctx
        .create_product
        .execute(
            "Smartphone".to_string(),
            Some(format!("smartphone-{}", Uuid::new_v4())),
            category_id,
            Some("A high-end smartphone".to_string()),
        )
        .await
        .unwrap();

    // 3. Verify in DB
    let row = sqlx::query("SELECT name FROM catalog.products WHERE id = $1")
        .bind(product_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let name: String = sqlx::Row::get(&row, "name");
    assert_eq!(name, "Smartphone");

    // 4. List Categories
    let categories = ctx.list_categories.execute().await.unwrap();
    assert!(!categories.is_empty());

    // 5. Update Product
    let updated_name = "Smartphone Pro".to_string();
    ctx.update_product
        .execute(
            product_id,
            updated_name.clone(),
            Some(format!("smartphone-pro-{}", Uuid::new_v4())),
            category_id,
            Some("Updated description".to_string()),
        )
        .await
        .unwrap();

    let row = sqlx::query("SELECT name FROM catalog.products WHERE id = $1")
        .bind(product_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let name: String = sqlx::Row::get(&row, "name");
    assert_eq!(name, updated_name);
}
