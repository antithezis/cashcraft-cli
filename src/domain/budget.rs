//! Budget domain model
//!
//! Tracks spending limits and progress per category and time period.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a budget for a specific category and month.
///
/// Budgets help track spending against planned limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    /// Month (1-12)
    pub month: u32,
    /// Year
    pub year: i32,
    /// Category this budget applies to
    pub category: String,
    /// Budgeted amount for the period
    pub amount: Decimal,
    /// Amount spent so far (calculated from transactions)
    pub spent: Decimal,
    /// Record creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Budget {
    /// Create a new budget for a category and month
    pub fn new(month: u32, year: i32, category: String, amount: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            month,
            year,
            category,
            amount,
            spent: Decimal::ZERO,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate remaining budget
    pub fn remaining(&self) -> Decimal {
        self.amount - self.spent
    }

    /// Calculate percentage used (0.0 to 100.0+)
    pub fn percentage_used(&self) -> f64 {
        if self.amount.is_zero() {
            return 0.0;
        }
        let ratio = self.spent / self.amount;
        ratio.try_into().unwrap_or(0.0) * 100.0
    }

    /// Check if over budget
    pub fn is_over_budget(&self) -> bool {
        self.spent > self.amount
    }
}
