use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stock {
    pub variant_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Supplier {
    pub id: Uuid,
    pub name: String,
    pub contact: String,
    pub location: String,
    pub status: String,
}
