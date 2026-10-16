use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user as the API shows them. There is deliberately no password hash here: responses are
/// built from this type, never from the domain `User`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserView {
    pub id: Uuid,
    pub email: String,
    pub role: String,
}

impl From<crate::domain::model::User> for UserView {
    fn from(user: crate::domain::model::User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            role: user.role,
        }
    }
}
