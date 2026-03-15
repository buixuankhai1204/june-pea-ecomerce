use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stock {
    pub variant_id: Uuid,
    pub quantity: i32,
}
