//! Budget service
//!
//! Business logic for managing budgets with progress tracking and warnings.

use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::budget::Budget;
use crate::domain::transaction::TransactionType;
use crate::error::Result;
use crate::repository::{
    budget_repo::BudgetRepository, Database, Repository, TransactionRepository,
};

/// Progress information for a budget.
#[derive(Debug, Clone)]
pub struct BudgetProgress {
    /// The budget
    pub budget: Budget,
    /// Amount spent
    pub spent: Decimal,
    /// Percentage used (0.0 to 100.0+)
    pub percentage: f64,
    /// Amount remaining (can be negative if over budget)
    pub remaining: Decimal,
    /// Whether the budget has reached warning threshold (80%)
    pub is_warning: bool,
    /// Whether the budget is over the limit
    pub is_over: bool,
}

/// Warning about a budget approaching or exceeding its limit.
#[derive(Debug, Clone)]
pub struct BudgetWarning {
    /// Category name
    pub category: String,
    /// Year
    pub year: i32,
    /// Month (1-12)
    pub month: u32,
    /// Budgeted amount
    pub budgeted: Decimal,
    /// Amount spent
    pub spent: Decimal,
    /// Percentage used
    pub percentage: f64,
    /// Warning severity level
    pub severity: WarningSeverity,
}

/// Severity level for budget warnings.
#[derive(Debug, Clone, PartialEq)]
pub enum WarningSeverity {
    /// Approaching limit (80-99%)
    Warning,
    /// At or over limit (100%+)
    Critical,
}

/// Service for managing budgets.
///
/// Provides CRUD operations, progress tracking, and budget warnings.
pub struct BudgetService<'a> {
    repo: BudgetRepository<'a>,
    transaction_repo: TransactionRepository<'a>,
}

impl<'a> BudgetService<'a> {
    /// Create a new BudgetService with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self {
            repo: BudgetRepository::new(db),
            transaction_repo: TransactionRepository::new(db),
        }
    }

    /// Create a new budget.
    ///
    /// # Arguments
    /// * `budget` - The budget to create
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn create(&self, budget: &Budget) -> Result<()> {
        self.repo.create(budget)
    }

    /// Get all budgets.
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - All budgets
    pub fn get_all(&self) -> Result<Vec<Budget>> {
        self.repo.get_all()
    }

    /// Get a budget by ID.
    ///
    /// # Arguments
    /// * `id` - The budget ID
    ///
    /// # Returns
    /// * `Result<Option<Budget>>` - The budget if found
    pub fn get_by_id(&self, id: &str) -> Result<Option<Budget>> {
        self.repo.get_by_id(id)
    }

    /// Update an existing budget.
    ///
    /// # Arguments
    /// * `budget` - The budget with updated values
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn update(&self, budget: &Budget) -> Result<()> {
        self.repo.update(budget)
    }

    /// Delete a budget.
    ///
    /// # Arguments
    /// * `id` - The budget ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn delete(&self, id: &str) -> Result<()> {
        self.repo.delete(id)
    }

    /// Get all budgets for a specific month (overrides only).
    ///
    /// Use `get_effective_budgets` to get templates + overrides merged.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - Budget overrides for the specified month
    pub fn get_month_budgets(&self, year: i32, month: u32) -> Result<Vec<Budget>> {
        self.repo.get_by_month(year, month)
    }

    /// Get effective budgets for a month (templates + overrides merged).
    ///
    /// For each category:
    /// - If a month-specific override exists, use it
    /// - Otherwise, use the template (if any)
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - Effective budgets for the month
    pub fn get_effective_budgets(&self, year: i32, month: u32) -> Result<Vec<Budget>> {
        self.repo.get_effective_budgets(year, month)
    }

    /// Get all budget templates.
    ///
    /// Templates apply to all months by default.
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - All budget templates
    pub fn get_templates(&self) -> Result<Vec<Budget>> {
        self.repo.get_templates()
    }

    /// Create a budget template for a category.
    ///
    /// Templates apply to all months by default.
    ///
    /// # Arguments
    /// * `category` - The category name
    /// * `amount` - The budget amount
    ///
    /// # Returns
    /// * `Result<Budget>` - The created template
    pub fn create_template(&self, category: &str, amount: Decimal) -> Result<Budget> {
        let template = Budget::new_template(category.to_string(), amount);
        self.repo.create(&template)?;
        Ok(template)
    }

    /// Create an override for a specific month.
    ///
    /// Overrides a template's amount for a specific month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    /// * `category` - The category
    /// * `amount` - The override amount
    ///
    /// # Returns
    /// * `Result<Budget>` - The created override
    pub fn create_override(
        &self,
        year: i32,
        month: u32,
        category: &str,
        amount: Decimal,
    ) -> Result<Budget> {
        let budget = Budget::new(month, year, category.to_string(), amount);
        self.repo.upsert(&budget)?;
        Ok(budget)
    }

    /// Get or create a budget for a specific month and category.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    /// * `category` - The category
    /// * `amount` - The budget amount (used only if creating)
    ///
    /// # Returns
    /// * `Result<Budget>` - The existing or newly created budget
    pub fn get_or_create(
        &self,
        year: i32,
        month: u32,
        category: &str,
        amount: Decimal,
    ) -> Result<Budget> {
        self.repo.get_or_create(year, month, category, amount)
    }

    /// Calculate budget progress for a specific month.
    ///
    /// Fetches actual spending from transactions and calculates progress
    /// for each budget category. Uses effective budgets (templates + overrides).
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<BudgetProgress>>` - Progress for each budget
    pub fn calculate_budget_progress(&self, year: i32, month: u32) -> Result<Vec<BudgetProgress>> {
        // Use effective budgets (templates + overrides merged)
        let budgets = self.get_effective_budgets(year, month)?;
        let transactions = self.transaction_repo.get_by_month(year, month)?;

        // Calculate spending by category from transactions
        let mut category_spending: HashMap<String, Decimal> = HashMap::new();
        for tx in transactions {
            if matches!(tx.transaction_type, TransactionType::Expense) {
                *category_spending
                    .entry(tx.category.clone())
                    .or_insert(Decimal::ZERO) += tx.amount.abs();
            }
        }

        let mut progress = Vec::new();
        for budget in budgets {
            let spent = category_spending
                .get(&budget.category)
                .cloned()
                .unwrap_or(Decimal::ZERO);

            let percentage = if budget.amount.is_zero() {
                0.0
            } else {
                let ratio: f64 = (spent / budget.amount).try_into().unwrap_or(0.0);
                ratio * 100.0
            };

            let remaining = budget.amount - spent;
            let is_warning = percentage >= 80.0 && percentage < 100.0;
            let is_over = percentage >= 100.0;

            progress.push(BudgetProgress {
                budget,
                spent,
                percentage,
                remaining,
                is_warning,
                is_over,
            });
        }

        Ok(progress)
    }

    /// Copy budgets from the previous month.
    ///
    /// Creates new budgets for the target month using the same categories
    /// and amounts as the previous month. Spent amounts are reset to zero.
    ///
    /// # Arguments
    /// * `year` - Target year
    /// * `month` - Target month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - The newly created budgets
    pub fn copy_from_previous_month(&self, year: i32, month: u32) -> Result<Vec<Budget>> {
        let (prev_year, prev_month) = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };

        self.repo
            .copy_from_month(prev_year, prev_month, year, month)
    }

    /// Check for budget warnings across all months.
    ///
    /// Returns warnings for budgets at 80%+ usage.
    ///
    /// # Returns
    /// * `Result<Vec<BudgetWarning>>` - All current budget warnings
    pub fn check_warnings(&self) -> Result<Vec<BudgetWarning>> {
        // Check current month
        let now = Utc::now();
        let year = now.date_naive().year();
        let month = now.date_naive().month();

        self.check_warnings_for_month(year, month)
    }

    /// Check for budget warnings for a specific month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<BudgetWarning>>` - Budget warnings for the month
    pub fn check_warnings_for_month(&self, year: i32, month: u32) -> Result<Vec<BudgetWarning>> {
        let progress = self.calculate_budget_progress(year, month)?;
        let mut warnings = Vec::new();

        for p in progress {
            if p.percentage >= 80.0 {
                let severity = if p.percentage >= 100.0 {
                    WarningSeverity::Critical
                } else {
                    WarningSeverity::Warning
                };

                warnings.push(BudgetWarning {
                    category: p.budget.category,
                    year: p.budget.year,
                    month: p.budget.month,
                    budgeted: p.budget.amount,
                    spent: p.spent,
                    percentage: p.percentage,
                    severity,
                });
            }
        }

        // Sort by percentage descending
        warnings.sort_by(|a, b| {
            b.percentage
                .partial_cmp(&a.percentage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(warnings)
    }

    /// Update spent amounts for all budgets in a month.
    ///
    /// Recalculates spent amounts from actual transactions.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn sync_spent_amounts(&self, year: i32, month: u32) -> Result<()> {
        let transactions = self.transaction_repo.get_by_month(year, month)?;
        let budgets = self.get_month_budgets(year, month)?;

        // Calculate spending by category
        let mut category_spending: HashMap<String, Decimal> = HashMap::new();
        for tx in transactions {
            if matches!(tx.transaction_type, TransactionType::Expense) {
                *category_spending
                    .entry(tx.category.clone())
                    .or_insert(Decimal::ZERO) += tx.amount.abs();
            }
        }

        // Update each budget's spent amount
        for budget in budgets {
            let spent = category_spending
                .get(&budget.category)
                .cloned()
                .unwrap_or(Decimal::ZERO);
            self.repo.update_spent(&budget.id.to_string(), spent)?;
        }

        Ok(())
    }

    /// Get budget summary for a month.
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<BudgetSummary>` - Summary of all budgets for the month
    pub fn get_month_summary(&self, year: i32, month: u32) -> Result<BudgetSummary> {
        let progress = self.calculate_budget_progress(year, month)?;

        let total_budgeted: Decimal = progress.iter().map(|p| p.budget.amount).sum();
        let total_spent: Decimal = progress.iter().map(|p| p.spent).sum();
        let budget_count = progress.len();
        let warning_count = progress.iter().filter(|p| p.is_warning).count();
        let over_count = progress.iter().filter(|p| p.is_over).count();

        Ok(BudgetSummary {
            year,
            month,
            total_budgeted,
            total_spent,
            total_remaining: total_budgeted - total_spent,
            budget_count,
            warning_count,
            over_count,
        })
    }

    /// Upsert a budget (insert or update).
    ///
    /// # Arguments
    /// * `budget` - The budget to upsert
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn upsert(&self, budget: &Budget) -> Result<()> {
        self.repo.upsert(budget)
    }
}

/// Summary of budgets for a month.
#[derive(Debug, Clone)]
pub struct BudgetSummary {
    /// Year
    pub year: i32,
    /// Month (1-12)
    pub month: u32,
    /// Total amount budgeted across all categories
    pub total_budgeted: Decimal,
    /// Total amount spent across all categories
    pub total_spent: Decimal,
    /// Total remaining (can be negative)
    pub total_remaining: Decimal,
    /// Number of budget categories
    pub budget_count: usize,
    /// Number of budgets at warning level (80-99%)
    pub warning_count: usize,
    /// Number of budgets over limit (100%+)
    pub over_count: usize,
}

use chrono::Datelike;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::transaction::Transaction;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn create_test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn create_test_budget() -> Budget {
        Budget::new(3, 2026, "Food".to_string(), dec!(600))
    }

    #[test]
    fn test_create_budget() {
        let db = create_test_db();
        let service = BudgetService::new(&db);
        let budget = create_test_budget();

        let result = service.create(&budget);
        assert!(result.is_ok());

        let retrieved = service.get_by_id(&budget.id.to_string()).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_budget_progress() {
        let db = create_test_db();
        let service = BudgetService::new(&db);

        // Create a budget
        let budget = Budget::new(3, 2026, "Food".to_string(), dec!(600));
        service.create(&budget).unwrap();

        // Create some transactions
        let tx_repo = TransactionRepository::new(&db);
        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Groceries".to_string(),
            dec!(200),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            "Restaurant".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );
        tx_repo.create(&tx1).unwrap();
        tx_repo.create(&tx2).unwrap();

        let progress = service.calculate_budget_progress(2026, 3).unwrap();
        assert_eq!(progress.len(), 1);
        assert_eq!(progress[0].spent, dec!(300));
        assert_eq!(progress[0].remaining, dec!(300));
        assert!((progress[0].percentage - 50.0).abs() < 0.01);
        assert!(!progress[0].is_warning);
        assert!(!progress[0].is_over);
    }

    #[test]
    fn test_budget_warnings() {
        let db = create_test_db();
        let service = BudgetService::new(&db);

        // Create a budget
        let budget = Budget::new(3, 2026, "Food".to_string(), dec!(100));
        service.create(&budget).unwrap();

        // Create transactions that exceed 80%
        let tx_repo = TransactionRepository::new(&db);
        let tx = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Groceries".to_string(),
            dec!(85),
            TransactionType::Expense,
            "Food".to_string(),
        );
        tx_repo.create(&tx).unwrap();

        let warnings = service.check_warnings_for_month(2026, 3).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, WarningSeverity::Warning);
    }

    #[test]
    fn test_critical_warning() {
        let db = create_test_db();
        let service = BudgetService::new(&db);

        // Create a budget
        let budget = Budget::new(3, 2026, "Food".to_string(), dec!(100));
        service.create(&budget).unwrap();

        // Create transactions that exceed 100%
        let tx_repo = TransactionRepository::new(&db);
        let tx = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Groceries".to_string(),
            dec!(120),
            TransactionType::Expense,
            "Food".to_string(),
        );
        tx_repo.create(&tx).unwrap();

        let warnings = service.check_warnings_for_month(2026, 3).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].severity, WarningSeverity::Critical);
    }

    #[test]
    fn test_copy_from_previous_month() {
        let db = create_test_db();
        let service = BudgetService::new(&db);

        // Create February budgets
        let budget1 = Budget::new(2, 2026, "Food".to_string(), dec!(600));
        let budget2 = Budget::new(2, 2026, "Transportation".to_string(), dec!(200));
        service.create(&budget1).unwrap();
        service.create(&budget2).unwrap();

        // Copy to March
        let copied = service.copy_from_previous_month(2026, 3).unwrap();
        assert_eq!(copied.len(), 2);

        let march = service.get_month_budgets(2026, 3).unwrap();
        assert_eq!(march.len(), 2);
    }

    #[test]
    fn test_month_summary() {
        let db = create_test_db();
        let service = BudgetService::new(&db);

        // Create budgets
        let budget1 = Budget::new(3, 2026, "Food".to_string(), dec!(600));
        let budget2 = Budget::new(3, 2026, "Transportation".to_string(), dec!(200));
        service.create(&budget1).unwrap();
        service.create(&budget2).unwrap();

        // Create transactions
        let tx_repo = TransactionRepository::new(&db);
        let tx = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Groceries".to_string(),
            dec!(300),
            TransactionType::Expense,
            "Food".to_string(),
        );
        tx_repo.create(&tx).unwrap();

        let summary = service.get_month_summary(2026, 3).unwrap();
        assert_eq!(summary.total_budgeted, dec!(800));
        assert_eq!(summary.total_spent, dec!(300));
        assert_eq!(summary.total_remaining, dec!(500));
        assert_eq!(summary.budget_count, 2);
    }
}
