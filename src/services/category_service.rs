//! Category service
//!
//! Aggregates categories from multiple sources for autocomplete suggestions.

use std::collections::HashSet;

use crate::error::Result;
use crate::repository::{
    budget_repo::BudgetRepository, Database, Repository, TransactionRepository,
};

/// Default categories from ExpenseCategory enum
pub const DEFAULT_CATEGORIES: &[&str] = &[
    "Housing",
    "Transportation",
    "Food",
    "Healthcare",
    "Entertainment",
    "Utilities",
    "Insurance",
    "Subscriptions",
    "PersonalCare",
    "Education",
    "Savings",
    "Debt",
];

/// Service for aggregating categories from all sources.
///
/// Provides category suggestions for autocomplete by combining:
/// - Categories from existing transactions
/// - Categories from existing budgets
/// - Predefined default categories
pub struct CategoryService<'a> {
    transaction_repo: TransactionRepository<'a>,
    budget_repo: BudgetRepository<'a>,
}

impl<'a> CategoryService<'a> {
    /// Create a new CategoryService with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self {
            transaction_repo: TransactionRepository::new(db),
            budget_repo: BudgetRepository::new(db),
        }
    }

    /// Get all unique categories from all sources.
    ///
    /// Returns categories sorted alphabetically with defaults first.
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - All unique categories
    pub fn get_all_categories(&self) -> Result<Vec<String>> {
        let mut categories: HashSet<String> = HashSet::new();

        // Add default categories first
        for cat in DEFAULT_CATEGORIES {
            categories.insert(cat.to_string());
        }

        // Add categories from transactions
        if let Ok(transactions) = self.transaction_repo.get_all() {
            for tx in transactions {
                if !tx.category.is_empty() {
                    categories.insert(tx.category);
                }
            }
        }

        // Add categories from budgets (both templates and overrides)
        if let Ok(budgets) = self.budget_repo.get_all() {
            for budget in budgets {
                if !budget.category.is_empty() {
                    categories.insert(budget.category);
                }
            }
        }

        // Convert to sorted vector
        let mut result: Vec<String> = categories.into_iter().collect();
        result.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        Ok(result)
    }

    /// Filter categories by a prefix (case-insensitive).
    ///
    /// # Arguments
    /// * `prefix` - The prefix to filter by
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - Matching categories
    pub fn filter_categories(&self, prefix: &str) -> Result<Vec<String>> {
        let all = self.get_all_categories()?;
        let prefix_lower = prefix.to_lowercase();

        let filtered: Vec<String> = all
            .into_iter()
            .filter(|cat| cat.to_lowercase().starts_with(&prefix_lower))
            .collect();

        Ok(filtered)
    }

    /// Search categories by substring (case-insensitive).
    ///
    /// Returns categories that contain the query anywhere in the name.
    ///
    /// # Arguments
    /// * `query` - The search query
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - Matching categories
    pub fn search_categories(&self, query: &str) -> Result<Vec<String>> {
        let all = self.get_all_categories()?;
        let query_lower = query.to_lowercase();

        // Prioritize prefix matches, then contains matches
        let mut prefix_matches: Vec<String> = Vec::new();
        let mut contains_matches: Vec<String> = Vec::new();

        for cat in all {
            let cat_lower = cat.to_lowercase();
            if cat_lower.starts_with(&query_lower) {
                prefix_matches.push(cat);
            } else if cat_lower.contains(&query_lower) {
                contains_matches.push(cat);
            }
        }

        // Combine with prefix matches first
        prefix_matches.extend(contains_matches);
        Ok(prefix_matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::budget::Budget;
    use crate::domain::transaction::{Transaction, TransactionType};
    use crate::repository::Repository;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn create_test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_default_categories() {
        let db = create_test_db();
        let service = CategoryService::new(&db);

        let categories = service.get_all_categories().unwrap();

        // Should include all defaults
        for default in DEFAULT_CATEGORIES {
            assert!(
                categories.contains(&default.to_string()),
                "Missing default: {}",
                default
            );
        }
    }

    #[test]
    fn test_categories_from_transactions() {
        let db = create_test_db();

        // Create a transaction with a custom category
        let tx = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            "Test".to_string(),
            dec!(100),
            TransactionType::Expense,
            "CustomCategory".to_string(),
        );
        let tx_repo = TransactionRepository::new(&db);
        tx_repo.create(&tx).unwrap();

        let service = CategoryService::new(&db);
        let categories = service.get_all_categories().unwrap();

        assert!(categories.contains(&"CustomCategory".to_string()));
    }

    #[test]
    fn test_categories_from_budgets() {
        let db = create_test_db();

        // Create a budget with a custom category
        let budget = Budget::new(3, 2026, "BudgetCategory".to_string(), dec!(500));
        let budget_repo = BudgetRepository::new(&db);
        budget_repo.create(&budget).unwrap();

        let service = CategoryService::new(&db);
        let categories = service.get_all_categories().unwrap();

        assert!(categories.contains(&"BudgetCategory".to_string()));
    }

    #[test]
    fn test_filter_categories() {
        let db = create_test_db();
        let service = CategoryService::new(&db);

        let filtered = service.filter_categories("foo").unwrap();
        assert!(filtered.contains(&"Food".to_string()));

        let filtered = service.filter_categories("Hou").unwrap();
        assert!(filtered.contains(&"Housing".to_string()));
    }

    #[test]
    fn test_search_categories() {
        let db = create_test_db();
        let service = CategoryService::new(&db);

        // Should find "Transportation" when searching for "port"
        let results = service.search_categories("port").unwrap();
        assert!(results.contains(&"Transportation".to_string()));

        // Prefix matches should come first
        let results = service.search_categories("ent").unwrap();
        assert!(results.contains(&"Entertainment".to_string()));
    }

    #[test]
    fn test_categories_sorted() {
        let db = create_test_db();
        let service = CategoryService::new(&db);

        let categories = service.get_all_categories().unwrap();

        // Check that categories are sorted (case-insensitive)
        let mut sorted = categories.clone();
        sorted.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        assert_eq!(categories, sorted);
    }
}
