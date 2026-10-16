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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::User;

    #[test]
    fn a_user_view_never_carries_the_password_hash() {
        let user = User::new("a@example.test".into(), "$argon2id$secret-hash".into());

        let json = serde_json::to_string(&UserView::from(user)).unwrap();

        assert!(!json.contains("argon2"), "{json}");
        assert!(!json.contains("password"), "{json}");
    }

    #[test]
    fn serializing_the_domain_user_leaves_the_hash_out_too() {
        let user = User::new("a@example.test".into(), "$argon2id$secret-hash".into());

        let json = serde_json::to_string(&user).unwrap();

        assert!(!json.contains("argon2"), "{json}");
    }
}
