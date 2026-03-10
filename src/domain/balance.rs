use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Represents a manually set opening balance for a specific month.
#[derive(Debug, Clone, PartialEq)]
pub struct MonthlyBalance {
    pub year: i32,
    pub month: u32,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MonthlyBalance {
    pub fn new(year: i32, month: u32, amount: Decimal) -> Self {
        Self {
            year,
            month,
            amount,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
