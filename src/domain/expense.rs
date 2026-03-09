//! Expense domain model
//!
//! Defines expenses with type categorization (fixed/variable) and budget priorities.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::income::Frequency;

/// Type of expense for budgeting purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpenseType {
    /// Fixed expenses that don't change month-to-month
    Fixed,
    /// Variable expenses that fluctuate
    Variable,
    /// One-time expenses
    OneTime,
}

/// Predefined expense categories for organization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpenseCategory {
    Housing,
    Transportation,
    Food,
    Healthcare,
    Entertainment,
    Utilities,
    Insurance,
    Subscriptions,
    PersonalCare,
    Education,
    Savings,
    Debt,
    Custom(String),
}

impl ExpenseCategory {
    /// Get string representation of the category
    pub fn as_str(&self) -> &str {
        match self {
            ExpenseCategory::Housing => "Housing",
            ExpenseCategory::Transportation => "Transportation",
            ExpenseCategory::Food => "Food",
            ExpenseCategory::Healthcare => "Healthcare",
            ExpenseCategory::Entertainment => "Entertainment",
            ExpenseCategory::Utilities => "Utilities",
            ExpenseCategory::Insurance => "Insurance",
            ExpenseCategory::Subscriptions => "Subscriptions",
            ExpenseCategory::PersonalCare => "PersonalCare",
            ExpenseCategory::Education => "Education",
            ExpenseCategory::Savings => "Savings",
            ExpenseCategory::Debt => "Debt",
            ExpenseCategory::Custom(s) => s,
        }
    }
}

/// Represents a recurring or one-time expense.
///
/// Expenses are referenced in the playground using `$variable_name` syntax.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expense {
    pub id: Uuid,
    /// Variable name for playground reference (e.g., "rent" -> $rent)
    pub variable_name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Amount per frequency period
    pub amount: Decimal,
    /// Whether this is a fixed, variable, or one-time expense
    pub expense_type: ExpenseType,
    /// How often this expense occurs
    pub frequency: Frequency,
    /// Category for grouping and reporting
    pub category: ExpenseCategory,
    /// Whether this expense is currently active
    pub is_active: bool,
    /// Whether this is an essential expense (for budgeting priorities)
    pub is_essential: bool,
    /// Day of month when due (1-31)
    pub due_day: Option<u8>,
    /// Additional notes
    pub notes: Option<String>,
    /// Record creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Expense {
    /// Create a new expense with default values
    pub fn new(
        variable_name: String,
        display_name: String,
        amount: Decimal,
        expense_type: ExpenseType,
        frequency: Frequency,
        category: ExpenseCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            variable_name: variable_name.to_lowercase(),
            display_name,
            amount,
            expense_type,
            frequency,
            category,
            is_active: true,
            is_essential: false,
            due_day: None,
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
