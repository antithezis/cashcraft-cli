//! Transaction service
//!
//! Business logic for managing transactions with aggregation and reporting.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::transaction::{Transaction, TransactionType};
use crate::error::Result;
use crate::repository::{Database, Repository, TransactionRepository};

/// Summary of transactions for a month.
#[derive(Debug, Clone)]
pub struct MonthSummary {
    /// Year of the summary
    pub year: i32,
    /// Month of the summary (1-12)
    pub month: u32,
    /// Total income for the month
    pub total_income: Decimal,
    /// Total expenses for the month
    pub total_expenses: Decimal,
    /// Net balance (income - expenses)
    pub net: Decimal,
    /// Number of income transactions
    pub income_count: usize,
    /// Number of expense transactions
    pub expense_count: usize,
}

/// Service for managing transactions.
///
/// Provides CRUD operations, date-based queries, and aggregation
/// for financial reporting.
pub struct TransactionService<'a> {
    repo: TransactionRepository<'a>,
}

impl<'a> TransactionService<'a> {
    /// Create a new TransactionService with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self {
            repo: TransactionRepository::new(db),
        }
    }

    /// Create a new transaction.
    ///
    /// # Arguments
    /// * `transaction` - The transaction to create
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn create(&self, transaction: &Transaction) -> Result<()> {
        self.repo.create(transaction)
    }

    /// Get all transactions.
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - All transactions
    pub fn get_all(&self) -> Result<Vec<Transaction>> {
        self.repo.get_all()
    }

    /// Get a transaction by ID.
    ///
    /// # Arguments
    /// * `id` - The transaction ID
    ///
    /// # Returns
    /// * `Result<Option<Transaction>>` - The transaction if found
    pub fn get_by_id(&self, id: &str) -> Result<Option<Transaction>> {
        self.repo.get_by_id(id)
    }

    /// Update an existing transaction.
    ///
    /// # Arguments
    /// * `transaction` - The transaction with updated values
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn update(&self, transaction: &Transaction) -> Result<()> {
        self.repo.update(transaction)
    }

    /// Delete a transaction.
    ///
    /// # Arguments
    /// * `id` - The transaction ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn delete(&self, id: &str) -> Result<()> {
        self.repo.delete(id)
    }

    /// Get transactions for a specific month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions for the specified month
    pub fn get_by_month(&self, year: i32, month: u32) -> Result<Vec<Transaction>> {
        self.repo.get_by_month(year, month)
    }

    /// Get transactions by category.
    ///
    /// # Arguments
    /// * `category` - The category to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions in the specified category
    pub fn get_by_category(&self, category: &str) -> Result<Vec<Transaction>> {
        self.repo.get_by_category(category)
    }

    /// Get transactions by type.
    ///
    /// # Arguments
    /// * `transaction_type` - The type to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions of the specified type
    pub fn get_by_type(&self, transaction_type: &TransactionType) -> Result<Vec<Transaction>> {
        self.repo.get_by_type(transaction_type)
    }

    /// Get transactions within a date range.
    ///
    /// # Arguments
    /// * `start` - Start date (inclusive)
    /// * `end` - End date (inclusive)
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions within the date range
    pub fn get_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<Transaction>> {
        self.repo.get_by_date_range(start, end)
    }

    /// Search transactions by description or notes.
    ///
    /// # Arguments
    /// * `query` - The search query (case-insensitive partial match)
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Matching transactions
    pub fn search(&self, query: &str) -> Result<Vec<Transaction>> {
        self.repo.search(query)
    }

    /// Calculate monthly summary for a specific month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<MonthSummary>` - Summary statistics for the month
    pub fn calculate_monthly_summary(&self, year: i32, month: u32) -> Result<MonthSummary> {
        let transactions = self.get_by_month(year, month)?;

        let mut total_income = Decimal::ZERO;
        let mut total_expenses = Decimal::ZERO;
        let mut income_count = 0;
        let mut expense_count = 0;

        for tx in &transactions {
            match tx.transaction_type {
                TransactionType::Income => {
                    total_income += tx.amount.abs();
                    income_count += 1;
                }
                TransactionType::Expense => {
                    total_expenses += tx.amount.abs();
                    expense_count += 1;
                }
                TransactionType::Transfer => {
                    // Transfers don't affect income/expense totals
                }
            }
        }

        Ok(MonthSummary {
            year,
            month,
            total_income,
            total_expenses,
            net: total_income - total_expenses,
            income_count,
            expense_count,
        })
    }

    /// Get category totals for a specific month.
    ///
    /// Returns expense totals grouped by category.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<HashMap<String, Decimal>>` - Category to total amount mapping
    pub fn get_category_totals(&self, year: i32, month: u32) -> Result<HashMap<String, Decimal>> {
        let transactions = self.get_by_month(year, month)?;
        let mut totals: HashMap<String, Decimal> = HashMap::new();

        for tx in transactions {
            // Only count expenses for category totals
            if matches!(tx.transaction_type, TransactionType::Expense) {
                *totals.entry(tx.category).or_insert(Decimal::ZERO) += tx.amount.abs();
            }
        }

        Ok(totals)
    }

    /// Get totals by transaction type for a specific month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<HashMap<String, Decimal>>` - Type to total amount mapping
    pub fn get_type_totals(&self, year: i32, month: u32) -> Result<HashMap<String, Decimal>> {
        let transactions = self.get_by_month(year, month)?;
        let mut totals: HashMap<String, Decimal> = HashMap::new();

        for tx in transactions {
            let type_name = match tx.transaction_type {
                TransactionType::Income => "Income",
                TransactionType::Expense => "Expense",
                TransactionType::Transfer => "Transfer",
            };
            *totals.entry(type_name.to_string()).or_insert(Decimal::ZERO) += tx.amount.abs();
        }

        Ok(totals)
    }

    /// Get recent transactions, limited to a specific count.
    ///
    /// # Arguments
    /// * `limit` - Maximum number of transactions to return
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Recent transactions
    pub fn get_recent(&self, limit: usize) -> Result<Vec<Transaction>> {
        let all = self.repo.get_all()?;
        Ok(all.into_iter().take(limit).collect())
    }

    /// Get recurring transactions.
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - All recurring transactions
    pub fn get_recurring(&self) -> Result<Vec<Transaction>> {
        self.repo.get_recurring()
    }

    /// Get transactions linked to a recurring source.
    ///
    /// # Arguments
    /// * `recurring_id` - The recurring source ID
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions linked to the source
    pub fn get_by_recurring_id(&self, recurring_id: &str) -> Result<Vec<Transaction>> {
        self.repo.get_by_recurring_id(recurring_id)
    }

    /// Calculate running balance for a date range.
    ///
    /// Returns transactions with cumulative balance.
    ///
    /// # Arguments
    /// * `start` - Start date (inclusive)
    /// * `end` - End date (inclusive)
    /// * `starting_balance` - Initial balance before the date range
    ///
    /// # Returns
    /// * `Result<Vec<(Transaction, Decimal)>>` - Transactions with running balance
    pub fn get_with_running_balance(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        starting_balance: Decimal,
    ) -> Result<Vec<(Transaction, Decimal)>> {
        let mut transactions = self.get_by_date_range(start, end)?;

        // Sort by date ascending for running balance calculation
        transactions.sort_by(|a, b| a.date.cmp(&b.date));

        let mut balance = starting_balance;
        let mut result = Vec::new();

        for tx in transactions {
            balance += tx.signed_amount();
            result.push((tx, balance));
        }

        // Reverse to match typical display order (most recent first)
        result.reverse();
        Ok(result)
    }

    /// Calculate average daily spending for a month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Decimal>` - Average daily spending
    pub fn average_daily_spending(&self, year: i32, month: u32) -> Result<Decimal> {
        let summary = self.calculate_monthly_summary(year, month)?;

        // Get number of days in the month
        let days_in_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(30);

        if days_in_month == 0 {
            return Ok(Decimal::ZERO);
        }

        Ok(summary.total_expenses / Decimal::from(days_in_month))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Database;
    use rust_decimal_macros::dec;

    fn create_test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn create_test_transaction() -> Transaction {
        Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap(),
            "Grocery Shopping".to_string(),
            dec!(127.45),
            TransactionType::Expense,
            "Food".to_string(),
        )
    }

    #[test]
    fn test_create_transaction() {
        let db = create_test_db();
        let service = TransactionService::new(&db);
        let tx = create_test_transaction();

        let result = service.create(&tx);
        assert!(result.is_ok());

        let retrieved = service.get_by_id(&tx.id.to_string()).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_monthly_summary() {
        let db = create_test_db();
        let service = TransactionService::new(&db);

        let income = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            "Salary".to_string(),
            dec!(4500),
            TransactionType::Income,
            "Salary".to_string(),
        );
        let expense1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rent".to_string(),
            dec!(1500),
            TransactionType::Expense,
            "Housing".to_string(),
        );
        let expense2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            "Groceries".to_string(),
            dec!(200),
            TransactionType::Expense,
            "Food".to_string(),
        );

        service.create(&income).unwrap();
        service.create(&expense1).unwrap();
        service.create(&expense2).unwrap();

        let summary = service.calculate_monthly_summary(2026, 3).unwrap();
        assert_eq!(summary.total_income, dec!(4500));
        assert_eq!(summary.total_expenses, dec!(1700));
        assert_eq!(summary.net, dec!(2800));
        assert_eq!(summary.income_count, 1);
        assert_eq!(summary.expense_count, 2);
    }

    #[test]
    fn test_category_totals() {
        let db = create_test_db();
        let service = TransactionService::new(&db);

        let expense1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rent".to_string(),
            dec!(1500),
            TransactionType::Expense,
            "Housing".to_string(),
        );
        let expense2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            "Groceries".to_string(),
            dec!(200),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let expense3 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            "Restaurant".to_string(),
            dec!(50),
            TransactionType::Expense,
            "Food".to_string(),
        );

        service.create(&expense1).unwrap();
        service.create(&expense2).unwrap();
        service.create(&expense3).unwrap();

        let totals = service.get_category_totals(2026, 3).unwrap();
        assert_eq!(totals.get("Housing"), Some(&dec!(1500)));
        assert_eq!(totals.get("Food"), Some(&dec!(250)));
    }

    #[test]
    fn test_search() {
        let db = create_test_db();
        let service = TransactionService::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap(),
            "Grocery Store".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            "Gas Station".to_string(),
            dec!(50),
            TransactionType::Expense,
            "Transportation".to_string(),
        );

        service.create(&tx1).unwrap();
        service.create(&tx2).unwrap();

        let results = service.search("grocery").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "Grocery Store");
    }

    #[test]
    fn test_running_balance() {
        let db = create_test_db();
        let service = TransactionService::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            "Salary".to_string(),
            dec!(1000),
            TransactionType::Income,
            "Salary".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rent".to_string(),
            dec!(500),
            TransactionType::Expense,
            "Housing".to_string(),
        );

        service.create(&tx1).unwrap();
        service.create(&tx2).unwrap();

        let balances = service
            .get_with_running_balance(
                NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
                dec!(100),
            )
            .unwrap();

        // Results are returned in reverse order (most recent first)
        assert_eq!(balances.len(), 2);
        // After income +1000 and expense -500, starting with 100
        // Final balance should be 600
        assert_eq!(balances[0].1, dec!(600));
    }

    #[test]
    fn test_average_daily_spending() {
        let db = create_test_db();
        let service = TransactionService::new(&db);

        // Create 310 in expenses for a 31-day month (March)
        let expense = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            "Expense".to_string(),
            dec!(310),
            TransactionType::Expense,
            "Test".to_string(),
        );

        service.create(&expense).unwrap();

        let avg = service.average_daily_spending(2026, 3).unwrap();
        assert_eq!(avg, dec!(10)); // 310 / 31 = 10
    }
}
