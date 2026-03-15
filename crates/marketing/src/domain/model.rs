use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::error::AppError;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct Coupon {
    pub id: Uuid,
    pub code: String,
    pub discount_amount: i64,
    pub max_uses: i32,
    pub current_uses: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl Coupon {
    pub fn new(code: String, discount_amount: i64, max_uses: i32) -> Result<Self, AppError> {
        if discount_amount <= 0 {
            return Err(AppError::Validation(
                "Discount amount must be positive".to_string(),
            ));
        }
        if max_uses < 0 {
            return Err(AppError::Validation(
                "Max uses cannot be negative".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            code,
            discount_amount,
            max_uses,
            current_uses: 0,
            is_active: true,
            created_at: Utc::now(),
        })
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && (self.max_uses == 0 || self.current_uses < self.max_uses)
    }

    pub fn increment_usage(&mut self) -> Result<(), AppError> {
        if !self.is_valid() {
            return Err(AppError::Validation(
                "Coupon is not valid or usage limit exceeded".to_string(),
            ));
        }
        self.current_uses += 1;
        Ok(())
    }

    pub fn deactivate(&mut self) -> Result<(), AppError> {
        if !self.is_active {
            return Err(AppError::Validation(
                "Coupon is already deactivated".to_string(),
            ));
        }
        self.is_active = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_coupon_is_valid() {
        let coupon = Coupon::new("SAVE10".to_string(), 1000, 5).unwrap();
        assert!(coupon.is_valid());
        assert_eq!(coupon.code, "SAVE10");
        assert_eq!(coupon.discount_amount, 1000);
        assert_eq!(coupon.max_uses, 5);
        assert_eq!(coupon.current_uses, 0);
        assert!(coupon.is_active);
    }

    #[test]
    fn new_coupon_zero_max_uses_means_unlimited() {
        let coupon = Coupon::new("UNLIMITED".to_string(), 1000, 0).unwrap();
        assert!(coupon.is_valid());
    }

    #[test]
    fn negative_discount_fails() {
        let res = Coupon::new("SAVE10".to_string(), -10, 5);
        assert!(res.is_err());
    }

    #[test]
    fn zero_discount_fails() {
        let res = Coupon::new("SAVE10".to_string(), 0, 5);
        assert!(res.is_err());
    }

    #[test]
    fn increment_usage_works() {
        let mut coupon = Coupon::new("SAVE10".to_string(), 1000, 1).unwrap();
        assert!(coupon.is_valid());
        assert!(coupon.increment_usage().is_ok());
        assert_eq!(coupon.current_uses, 1);
        assert!(!coupon.is_valid()); // max uses reached
        assert!(coupon.increment_usage().is_err());
    }

    #[test]
    fn increment_usage_unlimited() {
        let mut coupon = Coupon::new("SAVE10".to_string(), 1000, 0).unwrap();
        assert!(coupon.is_valid());
        assert!(coupon.increment_usage().is_ok());
        assert_eq!(coupon.current_uses, 1);
        assert!(coupon.is_valid());
    }
}
