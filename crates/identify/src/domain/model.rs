use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
}

impl User {
    pub fn new(email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            role: "customer".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Membership {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tier: String,
    pub points: i32,
    pub total_spent: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserMembership {
    pub user_id: Uuid,
    pub email: String,
    pub tier: String,
    pub points: i32,
    pub total_spent: rust_decimal::Decimal,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}
