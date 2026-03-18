use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StaffRole {
    Admin,
    Staff,
}

impl From<String> for StaffRole {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => StaffRole::Admin,
            _ => StaffRole::Staff,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StaffMember {
    pub id: Uuid,
    pub email: String,
    pub role: StaffRole,
}

impl From<crate::domain::model::User> for StaffMember {
    fn from(user: crate::domain::model::User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            role: StaffRole::from(user.role),
        }
    }
}
