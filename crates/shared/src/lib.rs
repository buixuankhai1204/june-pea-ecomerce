pub mod auth;
pub mod config;
pub mod database;
pub mod error;
pub mod infrastructure;
pub mod rules;
pub mod testing;

// Re-export các kiểu dữ liệu dùng chung (VD: Money, Pagination)
pub use error::AppError;
pub use rules::{ensure, BusinessRule};
