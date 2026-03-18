use crate::domain::model::{User, UserMembership};
use crate::domain::user_repository::UserRepository;
use shared::error::AppError;
use sqlx::PgPool;
use std::sync::Arc;

pub struct PostgresUserRepository {
    pool: Arc<PgPool>,
}

impl PostgresUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO identify.users (id, email, password_hash, role) VALUES ($1, $2, $3, $4)",
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, role FROM identify.users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&*self.pool)
        .await?;
        Ok(user)
    }

    // Tương tự cho find_by_id...
    async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, role FROM identify.users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await?;
        Ok(user)
    }

    async fn update_user(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE identify.users SET email = $1, password_hash = $2, role = $3 WHERE id = $4",
        )
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role)
        .bind(&user.id)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn list_users(&self) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, role FROM identify.users ORDER BY email",
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(users)
    }

    async fn list_staff(&self) -> Result<Vec<User>, AppError> {
        let staff = sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, role FROM identify.users WHERE role IN ('admin', 'staff') ORDER BY email",
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(staff)
    }

    async fn list_memberships(&self) -> Result<Vec<UserMembership>, AppError> {
        let rows = sqlx::query(
            "SELECT u.id as user_id, u.email, m.tier, m.points, m.total_spent, m.joined_at 
             FROM identify.users u 
             JOIN identify.memberships m ON u.id = m.user_id 
             ORDER BY m.joined_at DESC"
        )
        .fetch_all(&*self.pool)
        .await?;

        use sqlx::Row;
        Ok(rows.into_iter().map(|r| UserMembership {
            user_id: r.try_get("user_id").unwrap(),
            email: r.try_get("email").unwrap(),
            tier: r.try_get("tier").unwrap(),
            points: r.try_get("points").unwrap(),
            total_spent: r.try_get("total_spent").unwrap(),
            joined_at: r.try_get("joined_at").unwrap(),
        }).collect())
    }

    async fn delete_user(&self, id: uuid::Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM identify.users WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}
