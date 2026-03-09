//! Income domain model
//!
//! Defines income sources with frequency-based calculations for monthly equivalents.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Income frequency determines how often an income source pays out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    Yearly,
    OneTime,
}

impl Frequency {
    /// Convert amount to monthly equivalent
    pub fn to_monthly(&self, amount: Decimal) -> Decimal {
        match self {
            Frequency::Daily => amount * Decimal::from(30),
            Frequency::Weekly => amount * Decimal::from(4),
            Frequency::BiWeekly => amount * Decimal::from(2),
            Frequency::Monthly => amount,
            Frequency::Quarterly => amount / Decimal::from(3),
            Frequency::Yearly => amount / Decimal::from(12),
            Frequency::OneTime => Decimal::ZERO,
        }
    }
}

/// Represents a source of income (e.g., salary, freelance work, investments).
///
/// Income sources are referenced in the playground using `$variable_name` syntax.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncomeSource {
    pub id: Uuid,
    /// Variable name for playground reference (e.g., "salary" -> $salary)
    pub variable_name: String,
    /// Human-readable display name (e.g., "Primary Job")
    pub display_name: String,
    /// Amount per frequency period
    pub amount: Decimal,
    /// How often this income is received
    pub frequency: Frequency,
    /// Whether this income source is currently active
    pub is_active: bool,
    /// Optional category for grouping
    pub category: Option<String>,
    /// When this income source started
    pub start_date: Option<NaiveDate>,
    /// When this income source ends (for temporary income)
    pub end_date: Option<NaiveDate>,
    /// Additional notes
    pub notes: Option<String>,
    /// Record creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl IncomeSource {
    /// Create a new income source with default values
    pub fn new(
        variable_name: String,
        display_name: String,
        amount: Decimal,
        frequency: Frequency,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            variable_name: variable_name.to_lowercase(),
            display_name,
            amount,
            frequency,
            is_active: true,
            category: None,
            start_date: None,
            end_date: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get monthly amount based on frequency
    pub fn monthly_amount(&self) -> Decimal {
        self.frequency.to_monthly(self.amount)
    }
}
