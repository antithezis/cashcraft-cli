//! Transaction domain model
//!
//! Represents individual financial transactions for tracking and reporting.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of transaction for categorization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
}

/// Represents a single financial transaction.
///
/// Transactions track money in and out, with support for recurring
/// transactions linked to income sources or expenses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    /// Date of the transaction
    pub date: NaiveDate,
    /// Description of the transaction
    pub description: String,
    /// Amount (positive for income, negative for expense)
    pub amount: Decimal,
    /// Type of transaction
    pub transaction_type: TransactionType,
    /// Category for grouping and reporting
    pub category: String,
    /// Account this transaction belongs to (optional)
    pub account: Option<String>,
    /// Tags for flexible categorization
    pub tags: Vec<String>,
    /// Additional notes
    pub notes: Option<String>,
    /// Whether this is a recurring transaction
    pub is_recurring: bool,
    /// Reference to the recurring source (income/expense) if applicable
    pub recurring_id: Option<Uuid>,
    /// Record creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        date: NaiveDate,
        description: String,
        amount: Decimal,
        transaction_type: TransactionType,
        category: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            date,
            description,
            amount,
            transaction_type,
            category,
            account: None,
            tags: Vec::new(),
            notes: None,
            is_recurring: false,
            recurring_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns true if this is an income transaction
    pub fn is_income(&self) -> bool {
        matches!(self.transaction_type, TransactionType::Income)
    }

    /// Returns the signed amount (positive for income, negative for expense)
    pub fn signed_amount(&self) -> Decimal {
        match self.transaction_type {
            TransactionType::Income => self.amount.abs(),
            TransactionType::Expense => -self.amount.abs(),
            TransactionType::Transfer => self.amount,
        }
    }
}
