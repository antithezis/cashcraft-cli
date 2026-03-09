//! Budget domain model
//!
//! Tracks spending limits and progress per category and time period.
//!
//! ## Template System
//!
//! Budgets can be either:
//! - **Templates** (`is_template = true`): Apply to all months by default
//! - **Overrides** (`is_template = false`): Apply to a specific month only
//!
//! When viewing a month, the system shows template amounts unless
//! an override exists for that specific category/month.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a budget for a specific category and month.
///
/// Budgets help track spending against planned limits.
///
/// ## Template vs Override
///
/// - `is_template = true`: This budget is a template that applies to all months.
///   The `month` and `year` fields are ignored (set to 0).
/// - `is_template = false`: This budget is an override for a specific month/year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    /// Month (1-12, or 0 for templates)
    pub month: u32,
    /// Year (or 0 for templates)
    pub year: i32,
    /// Category this budget applies to
    pub category: String,
    /// Budgeted amount for the period
    pub amount: Decimal,
    /// Amount spent so far (calculated from transactions)
    pub spent: Decimal,
    /// If true, this is a template that applies to all months
    pub is_template: bool,
    /// Record creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Budget {
    /// Create a new budget for a category and month (as an override)
    pub fn new(month: u32, year: i32, category: String, amount: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            month,
            year,
            category,
            amount,
            spent: Decimal::ZERO,
            is_template: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new budget template that applies to all months
    pub fn new_template(category: String, amount: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            month: 0,
            year: 0,
            category,
            amount,
            spent: Decimal::ZERO,
            is_template: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create an override from a template for a specific month
    pub fn override_for_month(&self, month: u32, year: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            month,
            year,
            category: self.category.clone(),
            amount: self.amount,
            spent: Decimal::ZERO,
            is_template: false,
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
