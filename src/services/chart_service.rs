//! Chart service
//!
//! Business logic for generating chart data for visualization.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::transaction::TransactionType;
use crate::error::Result;
use crate::repository::{Database, ExpenseRepository, IncomeRepository, TransactionRepository};

/// Data point for income vs expenses chart.
#[derive(Debug, Clone)]
pub struct IncomeExpensePoint {
    /// First day of the month
    pub date: NaiveDate,
    /// Total income for the month
    pub income: Decimal,
    /// Total expenses for the month
    pub expenses: Decimal,
    /// Net balance (income - expenses)
    pub net: Decimal,
}

/// Data point for category breakdown.
#[derive(Debug, Clone)]
pub struct CategoryBreakdown {
    /// Category name
    pub category: String,
    /// Total amount spent
    pub amount: Decimal,
    /// Percentage of total
    pub percentage: f64,
}

/// Data point for savings trend.
#[derive(Debug, Clone)]
pub struct SavingsPoint {
    /// First day of the month
    pub date: NaiveDate,
    /// Monthly savings (income - expenses)
    pub monthly_savings: Decimal,
    /// Cumulative savings up to this point
    pub cumulative_savings: Decimal,
}

/// Data point for expense trend by category.
#[derive(Debug, Clone)]
pub struct CategoryTrendPoint {
    /// First day of the month
    pub date: NaiveDate,
    /// Category name
    pub category: String,
    /// Amount spent
    pub amount: Decimal,
}

/// Service for generating chart data.
///
/// Provides data formatted for various chart visualizations in the TUI.
pub struct ChartService<'a> {
    income_repo: IncomeRepository<'a>,
    expense_repo: ExpenseRepository<'a>,
    transaction_repo: TransactionRepository<'a>,
}

impl<'a> ChartService<'a> {
    /// Create a new ChartService with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self {
            income_repo: IncomeRepository::new(db),
            expense_repo: ExpenseRepository::new(db),
            transaction_repo: TransactionRepository::new(db),
        }
    }

    /// Get income vs expense data for the last N months.
    ///
    /// Returns data sorted by date ascending.
    ///
    /// # Arguments
    /// * `months` - Number of months to include
    ///
    /// # Returns
    /// * `Result<Vec<IncomeExpensePoint>>` - Monthly income/expense data
    pub fn income_vs_expenses(&self, months: usize) -> Result<Vec<IncomeExpensePoint>> {
        let mut result = Vec::new();
        let today = chrono::Utc::now().date_naive();

        for i in (0..months).rev() {
            let (year, month) = self.month_offset(today.year(), today.month(), i);
            let transactions = self.transaction_repo.get_by_month(year, month)?;

            let mut income = Decimal::ZERO;
            let mut expenses = Decimal::ZERO;

            for tx in transactions {
                match tx.transaction_type {
                    TransactionType::Income => income += tx.amount.abs(),
                    TransactionType::Expense => expenses += tx.amount.abs(),
                    TransactionType::Transfer => {}
                }
            }

            let date = NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, 1, 1).unwrap());

            result.push(IncomeExpensePoint {
                date,
                income,
                expenses,
                net: income - expenses,
            });
        }

        Ok(result)
    }

    /// Get category breakdown for a specific month.
    ///
    /// Returns categories sorted by amount descending.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<CategoryBreakdown>>` - Category breakdown data
    pub fn category_breakdown(&self, year: i32, month: u32) -> Result<Vec<CategoryBreakdown>> {
        let transactions = self.transaction_repo.get_by_month(year, month)?;

        // Calculate totals by category (expenses only)
        let mut category_totals: HashMap<String, Decimal> = HashMap::new();
        let mut total = Decimal::ZERO;

        for tx in transactions {
            if matches!(tx.transaction_type, TransactionType::Expense) {
                let amount = tx.amount.abs();
                *category_totals
                    .entry(tx.category.clone())
                    .or_insert(Decimal::ZERO) += amount;
                total += amount;
            }
        }

        // Convert to breakdown with percentages
        let mut breakdown: Vec<CategoryBreakdown> = category_totals
            .into_iter()
            .map(|(category, amount)| {
                let percentage = if total.is_zero() {
                    0.0
                } else {
                    let ratio: f64 = (amount / total).try_into().unwrap_or(0.0);
                    ratio * 100.0
                };

                CategoryBreakdown {
                    category,
                    amount,
                    percentage,
                }
            })
            .collect();

        // Sort by amount descending
        breakdown.sort_by(|a, b| b.amount.cmp(&a.amount));

        Ok(breakdown)
    }

    /// Get savings trend for the last N months.
    ///
    /// Returns data sorted by date ascending with cumulative savings.
    ///
    /// # Arguments
    /// * `months` - Number of months to include
    ///
    /// # Returns
    /// * `Result<Vec<SavingsPoint>>` - Monthly and cumulative savings data
    pub fn savings_trend(&self, months: usize) -> Result<Vec<SavingsPoint>> {
        let income_expense = self.income_vs_expenses(months)?;
        let mut cumulative = Decimal::ZERO;
        let mut result = Vec::new();

        for point in income_expense {
            cumulative += point.net;
            result.push(SavingsPoint {
                date: point.date,
                monthly_savings: point.net,
                cumulative_savings: cumulative,
            });
        }

        Ok(result)
    }

    /// Get expense trend by category for the last N months.
    ///
    /// Returns data for each category over time.
    ///
    /// # Arguments
    /// * `months` - Number of months to include
    /// * `top_n` - Number of top categories to include (by total amount)
    ///
    /// # Returns
    /// * `Result<HashMap<String, Vec<CategoryTrendPoint>>>` - Trend data by category
    pub fn category_trend(
        &self,
        months: usize,
        top_n: usize,
    ) -> Result<HashMap<String, Vec<CategoryTrendPoint>>> {
        let today = chrono::Utc::now().date_naive();
        let mut category_totals: HashMap<String, Decimal> = HashMap::new();
        let mut month_data: Vec<(NaiveDate, HashMap<String, Decimal>)> = Vec::new();

        // Collect data for all months
        for i in (0..months).rev() {
            let (year, month) = self.month_offset(today.year(), today.month(), i);
            let transactions = self.transaction_repo.get_by_month(year, month)?;

            let mut month_categories: HashMap<String, Decimal> = HashMap::new();

            for tx in transactions {
                if matches!(tx.transaction_type, TransactionType::Expense) {
                    let amount = tx.amount.abs();
                    *month_categories
                        .entry(tx.category.clone())
                        .or_insert(Decimal::ZERO) += amount;
                    *category_totals
                        .entry(tx.category.clone())
                        .or_insert(Decimal::ZERO) += amount;
                }
            }

            let date = NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(year, 1, 1).unwrap());

            month_data.push((date, month_categories));
        }

        // Find top N categories by total amount
        let mut category_list: Vec<(String, Decimal)> = category_totals.into_iter().collect();
        category_list.sort_by(|a, b| b.1.cmp(&a.1));
        let top_categories: Vec<String> = category_list
            .into_iter()
            .take(top_n)
            .map(|(cat, _)| cat)
            .collect();

        // Build trend data for top categories
        let mut result: HashMap<String, Vec<CategoryTrendPoint>> = HashMap::new();

        for category in &top_categories {
            let mut trend = Vec::new();
            for (date, month_cats) in &month_data {
                let amount = month_cats.get(category).cloned().unwrap_or(Decimal::ZERO);
                trend.push(CategoryTrendPoint {
                    date: *date,
                    category: category.clone(),
                    amount,
                });
            }
            result.insert(category.clone(), trend);
        }

        Ok(result)
    }

    /// Get projected monthly balance based on defined income and expenses.
    ///
    /// Uses the income_sources and expenses tables to calculate projected
    /// monthly balance (not actual transactions).
    ///
    /// # Returns
    /// * `Result<(Decimal, Decimal, Decimal)>` - (total_income, total_expenses, balance)
    pub fn projected_monthly_balance(&self) -> Result<(Decimal, Decimal, Decimal)> {
        let active_income = self.income_repo.get_active()?;
        let active_expenses = self.expense_repo.get_active()?;

        let total_income: Decimal = active_income.iter().map(|i| i.monthly_amount()).sum();
        let total_expenses: Decimal = active_expenses.iter().map(|e| e.monthly_amount()).sum();

        Ok((total_income, total_expenses, total_income - total_expenses))
    }

    /// Get expense breakdown by type (Fixed, Variable, OneTime).
    ///
    /// Uses defined expenses, not transactions.
    ///
    /// # Returns
    /// * `Result<Vec<CategoryBreakdown>>` - Breakdown by expense type
    pub fn expense_type_breakdown(&self) -> Result<Vec<CategoryBreakdown>> {
        let active_expenses = self.expense_repo.get_active()?;

        let mut type_totals: HashMap<String, Decimal> = HashMap::new();
        let mut total = Decimal::ZERO;

        for expense in active_expenses {
            let type_name = match expense.expense_type {
                crate::domain::expense::ExpenseType::Fixed => "Fixed",
                crate::domain::expense::ExpenseType::Variable => "Variable",
                crate::domain::expense::ExpenseType::OneTime => "One-Time",
            };
            let amount = expense.monthly_amount();
            *type_totals
                .entry(type_name.to_string())
                .or_insert(Decimal::ZERO) += amount;
            total += amount;
        }

        let mut breakdown: Vec<CategoryBreakdown> = type_totals
            .into_iter()
            .map(|(category, amount)| {
                let percentage = if total.is_zero() {
                    0.0
                } else {
                    let ratio: f64 = (amount / total).try_into().unwrap_or(0.0);
                    ratio * 100.0
                };

                CategoryBreakdown {
                    category,
                    amount,
                    percentage,
                }
            })
            .collect();

        breakdown.sort_by(|a, b| b.amount.cmp(&a.amount));

        Ok(breakdown)
    }

    /// Get daily spending for a month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<(NaiveDate, Decimal)>>` - Daily spending amounts
    pub fn daily_spending(&self, year: i32, month: u32) -> Result<Vec<(NaiveDate, Decimal)>> {
        let transactions = self.transaction_repo.get_by_month(year, month)?;

        let mut daily: HashMap<NaiveDate, Decimal> = HashMap::new();

        for tx in transactions {
            if matches!(tx.transaction_type, TransactionType::Expense) {
                *daily.entry(tx.date).or_insert(Decimal::ZERO) += tx.amount.abs();
            }
        }

        let mut result: Vec<(NaiveDate, Decimal)> = daily.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result)
    }

    /// Calculate month offset from current date.
    fn month_offset(&self, year: i32, month: u32, offset_back: usize) -> (i32, u32) {
        let total_months = year * 12 + month as i32 - 1 - offset_back as i32;
        let new_year = total_months / 12;
        let new_month = (total_months % 12) + 1;
        (new_year, new_month as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::transaction::Transaction;
    use crate::repository::Repository;
    use rust_decimal_macros::dec;

    fn create_test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_category_breakdown() {
        let db = create_test_db();
        let service = ChartService::new(&db);
        let tx_repo = TransactionRepository::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rent".to_string(),
            dec!(1500),
            TransactionType::Expense,
            "Housing".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            "Groceries".to_string(),
            dec!(500),
            TransactionType::Expense,
            "Food".to_string(),
        );

        tx_repo.create(&tx1).unwrap();
        tx_repo.create(&tx2).unwrap();

        let breakdown = service.category_breakdown(2026, 3).unwrap();
        assert_eq!(breakdown.len(), 2);

        // Should be sorted by amount descending
        assert_eq!(breakdown[0].category, "Housing");
        assert_eq!(breakdown[0].amount, dec!(1500));
        assert!((breakdown[0].percentage - 75.0).abs() < 0.01);

        assert_eq!(breakdown[1].category, "Food");
        assert_eq!(breakdown[1].amount, dec!(500));
        assert!((breakdown[1].percentage - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_income_vs_expenses() {
        let db = create_test_db();
        let service = ChartService::new(&db);
        let tx_repo = TransactionRepository::new(&db);

        // Create transactions for current month
        let today = chrono::Utc::now().date_naive();
        let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

        let income = Transaction::new(
            first_of_month,
            "Salary".to_string(),
            dec!(4500),
            TransactionType::Income,
            "Salary".to_string(),
        );
        let expense = Transaction::new(
            first_of_month,
            "Rent".to_string(),
            dec!(1500),
            TransactionType::Expense,
            "Housing".to_string(),
        );

        tx_repo.create(&income).unwrap();
        tx_repo.create(&expense).unwrap();

        let data = service.income_vs_expenses(1).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].income, dec!(4500));
        assert_eq!(data[0].expenses, dec!(1500));
        assert_eq!(data[0].net, dec!(3000));
    }

    #[test]
    fn test_savings_trend() {
        let db = create_test_db();
        let service = ChartService::new(&db);
        let tx_repo = TransactionRepository::new(&db);

        let today = chrono::Utc::now().date_naive();
        let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

        let income = Transaction::new(
            first_of_month,
            "Salary".to_string(),
            dec!(4500),
            TransactionType::Income,
            "Salary".to_string(),
        );
        let expense = Transaction::new(
            first_of_month,
            "Rent".to_string(),
            dec!(1500),
            TransactionType::Expense,
            "Housing".to_string(),
        );

        tx_repo.create(&income).unwrap();
        tx_repo.create(&expense).unwrap();

        let trend = service.savings_trend(1).unwrap();
        assert_eq!(trend.len(), 1);
        assert_eq!(trend[0].monthly_savings, dec!(3000));
        assert_eq!(trend[0].cumulative_savings, dec!(3000));
    }

    #[test]
    fn test_projected_balance() {
        let db = create_test_db();
        let service = ChartService::new(&db);

        use crate::domain::expense::{Expense, ExpenseCategory, ExpenseType};
        use crate::domain::income::{Frequency, IncomeSource};

        let income_repo = IncomeRepository::new(&db);
        let expense_repo = ExpenseRepository::new(&db);

        let income = IncomeSource::new(
            "salary".to_string(),
            "Primary Job".to_string(),
            dec!(4500),
            Frequency::Monthly,
        );
        let expense = Expense::new(
            "rent".to_string(),
            "Apartment".to_string(),
            dec!(1500),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        );

        income_repo.create(&income).unwrap();
        expense_repo.create(&expense).unwrap();

        let (total_income, total_expenses, balance) = service.projected_monthly_balance().unwrap();

        assert_eq!(total_income, dec!(4500));
        assert_eq!(total_expenses, dec!(1500));
        assert_eq!(balance, dec!(3000));
    }

    #[test]
    fn test_daily_spending() {
        let db = create_test_db();
        let service = ChartService::new(&db);
        let tx_repo = TransactionRepository::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Groceries".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Coffee".to_string(),
            dec!(5),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx3 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            "Gas".to_string(),
            dec!(50),
            TransactionType::Expense,
            "Transportation".to_string(),
        );

        tx_repo.create(&tx1).unwrap();
        tx_repo.create(&tx2).unwrap();
        tx_repo.create(&tx3).unwrap();

        let daily = service.daily_spending(2026, 3).unwrap();
        assert_eq!(daily.len(), 2);

        // Should be sorted by date
        assert_eq!(daily[0].0, NaiveDate::from_ymd_opt(2026, 3, 5).unwrap());
        assert_eq!(daily[0].1, dec!(105)); // 100 + 5

        assert_eq!(daily[1].0, NaiveDate::from_ymd_opt(2026, 3, 10).unwrap());
        assert_eq!(daily[1].1, dec!(50));
    }

    #[test]
    fn test_month_offset() {
        let db = create_test_db();
        let service = ChartService::new(&db);

        // March 2026 - 0 months back = March 2026
        assert_eq!(service.month_offset(2026, 3, 0), (2026, 3));

        // March 2026 - 1 month back = February 2026
        assert_eq!(service.month_offset(2026, 3, 1), (2026, 2));

        // March 2026 - 3 months back = December 2025
        assert_eq!(service.month_offset(2026, 3, 3), (2025, 12));

        // March 2026 - 12 months back = March 2025
        assert_eq!(service.month_offset(2026, 3, 12), (2025, 3));
    }
}
