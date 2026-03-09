//! Budget repository
//!
//! Provides CRUD operations for Budget entities using SQLite.
//!
//! ## Template System
//!
//! Budgets can be either templates (`is_template = true`) that apply to all months,
//! or overrides (`is_template = false`) that apply to a specific month/year.
//!
//! Key methods:
//! - `get_templates()` - Get all budget templates
//! - `get_effective_budgets()` - Get effective budgets for a month (templates + overrides)
//! - `get_by_month()` - Get only the explicit overrides for a month

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use super::{Database, Repository};
use crate::domain::budget::Budget;
use crate::error::Result;

/// Repository for managing budgets in the database.
pub struct BudgetRepository<'a> {
    db: &'a Database,
}

impl<'a> BudgetRepository<'a> {
    /// Create a new BudgetRepository with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Get all budgets for a specific month (overrides only, not templates).
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - All budget overrides for the specified month
    pub fn get_by_month(&self, year: i32, month: u32) -> Result<Vec<Budget>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, month, year, category, amount, spent, is_template, created_at, updated_at
             FROM budgets 
             WHERE year = ?1 AND month = ?2 AND is_template = 0
             ORDER BY category",
        )?;

        let rows = stmt.query_map(params![year, month as i32], |row| {
            Self::row_to_budget(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut budgets = Vec::new();
        for row in rows {
            budgets.push(row?);
        }

        Ok(budgets)
    }

    /// Get all budget templates.
    ///
    /// Templates have `is_template = true` and apply to all months by default.
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - All budget templates
    pub fn get_templates(&self) -> Result<Vec<Budget>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, month, year, category, amount, spent, is_template, created_at, updated_at
             FROM budgets 
             WHERE is_template = 1
             ORDER BY category",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_budget(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut budgets = Vec::new();
        for row in rows {
            budgets.push(row?);
        }

        Ok(budgets)
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
        let templates = self.get_templates()?;
        let overrides = self.get_by_month(year, month)?;

        // Build a map of category -> override
        let override_map: HashMap<String, Budget> = overrides
            .into_iter()
            .map(|b| (b.category.clone(), b))
            .collect();

        let mut effective = Vec::new();

        // Add all templates, but use override if one exists
        for template in templates {
            if let Some(override_budget) = override_map.get(&template.category) {
                effective.push(override_budget.clone());
            } else {
                // Create a virtual budget from template for this month
                let mut virtual_budget = template.override_for_month(month, year);
                virtual_budget.id = template.id; // Keep template ID for reference
                virtual_budget.is_template = true; // Mark as coming from template
                effective.push(virtual_budget);
            }
        }

        // Add any overrides for categories not in templates
        for (category, override_budget) in override_map {
            if !effective.iter().any(|b| b.category == category) {
                effective.push(override_budget);
            }
        }

        // Sort by category
        effective.sort_by(|a, b| a.category.cmp(&b.category));

        Ok(effective)
    }

    /// Get a budget for a specific month and category (override only).
    ///
    /// # Arguments
    /// * `year` - The year
    /// * `month` - The month (1-12)
    /// * `category` - The category
    ///
    /// # Returns
    /// * `Result<Option<Budget>>` - The budget if found
    pub fn get_by_month_category(
        &self,
        year: i32,
        month: u32,
        category: &str,
    ) -> Result<Option<Budget>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, month, year, category, amount, spent, is_template, created_at, updated_at
             FROM budgets 
             WHERE year = ?1 AND month = ?2 AND category = ?3 AND is_template = 0",
        )?;

        let mut rows = stmt.query(params![year, month as i32, category])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_budget(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get a template for a specific category.
    ///
    /// # Arguments
    /// * `category` - The category
    ///
    /// # Returns
    /// * `Result<Option<Budget>>` - The template if found
    pub fn get_template_by_category(&self, category: &str) -> Result<Option<Budget>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, month, year, category, amount, spent, is_template, created_at, updated_at
             FROM budgets 
             WHERE category = ?1 AND is_template = 1",
        )?;

        let mut rows = stmt.query(params![category])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_budget(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get or create a budget for a specific month and category.
    ///
    /// If the budget exists, returns it. Otherwise, creates a new budget
    /// with the specified amount and returns it.
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
        if let Some(existing) = self.get_by_month_category(year, month, category)? {
            return Ok(existing);
        }

        let budget = Budget::new(month, year, category.to_string(), amount);
        self.create(&budget)?;
        Ok(budget)
    }

    /// Upsert a budget (insert or update).
    ///
    /// If a budget exists for the given month/year/category/is_template, updates it.
    /// Otherwise, creates a new budget.
    ///
    /// # Arguments
    /// * `budget` - The budget to upsert
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn upsert(&self, budget: &Budget) -> Result<()> {
        self.db.conn.execute(
            "INSERT INTO budgets (id, month, year, category, amount, spent, is_template, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(month, year, category) DO UPDATE SET
                 amount = excluded.amount,
                 spent = excluded.spent,
                 is_template = excluded.is_template,
                 updated_at = excluded.updated_at",
            params![
                budget.id.to_string(),
                budget.month as i32,
                budget.year,
                budget.category,
                budget.amount.to_string(),
                budget.spent.to_string(),
                budget.is_template as i32,
                budget.created_at.to_rfc3339(),
                budget.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Update the spent amount for a budget.
    ///
    /// # Arguments
    /// * `id` - The budget ID
    /// * `spent` - The new spent amount
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn update_spent(&self, id: &str, spent: Decimal) -> Result<()> {
        self.db.conn.execute(
            "UPDATE budgets SET spent = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, spent.to_string(), Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Get budgets for a year (overrides only, not templates).
    ///
    /// # Arguments
    /// * `year` - The year
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - All budget overrides for the year
    pub fn get_by_year(&self, year: i32) -> Result<Vec<Budget>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, month, year, category, amount, spent, is_template, created_at, updated_at
             FROM budgets 
             WHERE year = ?1 AND is_template = 0
             ORDER BY month, category",
        )?;

        let rows = stmt.query_map(params![year], |row| {
            Self::row_to_budget(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut budgets = Vec::new();
        for row in rows {
            budgets.push(row?);
        }

        Ok(budgets)
    }

    /// Copy budgets from one month to another.
    ///
    /// Copies all budget categories and amounts from the source month
    /// to the target month. Spent amounts are reset to zero.
    ///
    /// # Arguments
    /// * `from_year` - Source year
    /// * `from_month` - Source month
    /// * `to_year` - Target year
    /// * `to_month` - Target month
    ///
    /// # Returns
    /// * `Result<Vec<Budget>>` - The newly created budgets
    pub fn copy_from_month(
        &self,
        from_year: i32,
        from_month: u32,
        to_year: i32,
        to_month: u32,
    ) -> Result<Vec<Budget>> {
        let source_budgets = self.get_by_month(from_year, from_month)?;
        let mut created_budgets = Vec::new();

        for source in source_budgets {
            let new_budget = Budget::new(to_month, to_year, source.category, source.amount);
            self.upsert(&new_budget)?;
            created_budgets.push(new_budget);
        }

        Ok(created_budgets)
    }

    /// Convert a database row to a Budget.
    /// Expected column order: id, month, year, category, amount, spent, is_template, created_at, updated_at
    fn row_to_budget(row: &Row<'_>) -> Result<Budget> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let month: i32 = row.get(1)?;
        let year: i32 = row.get(2)?;

        let amount_str: String = row.get(4)?;
        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let spent_str: String = row.get(5)?;
        let spent = Decimal::from_str(&spent_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let is_template: i32 = row.get(6)?;

        let created_at_str: String = row.get(7)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let updated_at_str: String = row.get(8)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        Ok(Budget {
            id,
            month: month as u32,
            year,
            category: row.get(3)?,
            amount,
            spent,
            is_template: is_template != 0,
            created_at,
            updated_at,
        })
    }
}

impl<'a> Repository<Budget> for BudgetRepository<'a> {
    fn create(&self, item: &Budget) -> Result<()> {
        self.db.conn.execute(
            "INSERT INTO budgets 
             (id, month, year, category, amount, spent, is_template, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                item.id.to_string(),
                item.month as i32,
                item.year,
                item.category,
                item.amount.to_string(),
                item.spent.to_string(),
                item.is_template as i32,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Budget>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, month, year, category, amount, spent, is_template, created_at, updated_at
             FROM budgets 
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_budget(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_all(&self) -> Result<Vec<Budget>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, month, year, category, amount, spent, is_template, created_at, updated_at
             FROM budgets 
             ORDER BY is_template DESC, year DESC, month DESC, category",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_budget(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut budgets = Vec::new();
        for row in rows {
            budgets.push(row?);
        }

        Ok(budgets)
    }

    fn update(&self, item: &Budget) -> Result<()> {
        self.db.conn.execute(
            "UPDATE budgets 
             SET month = ?2, year = ?3, category = ?4, amount = ?5, spent = ?6, is_template = ?7, updated_at = ?8
             WHERE id = ?1",
            params![
                item.id.to_string(),
                item.month as i32,
                item.year,
                item.category,
                item.amount.to_string(),
                item.spent.to_string(),
                item.is_template as i32,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.db
            .conn
            .execute("DELETE FROM budgets WHERE id = ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn create_test_budget() -> Budget {
        Budget::new(3, 2026, "Food".to_string(), dec!(600))
    }

    #[test]
    fn test_create_and_get_by_id() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);
        let budget = create_test_budget();

        repo.create(&budget).unwrap();

        let retrieved = repo.get_by_id(&budget.id.to_string()).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.month, 3);
        assert_eq!(retrieved.year, 2026);
        assert_eq!(retrieved.category, "Food");
        assert_eq!(retrieved.amount, dec!(600));
    }

    #[test]
    fn test_get_by_month() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);

        let budget1 = Budget::new(3, 2026, "Food".to_string(), dec!(600));
        let budget2 = Budget::new(3, 2026, "Transportation".to_string(), dec!(200));
        let budget3 = Budget::new(2, 2026, "Food".to_string(), dec!(550));

        repo.create(&budget1).unwrap();
        repo.create(&budget2).unwrap();
        repo.create(&budget3).unwrap();

        let march = repo.get_by_month(2026, 3).unwrap();
        assert_eq!(march.len(), 2);

        let feb = repo.get_by_month(2026, 2).unwrap();
        assert_eq!(feb.len(), 1);
    }

    #[test]
    fn test_get_by_month_category() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);

        let budget = Budget::new(3, 2026, "Food".to_string(), dec!(600));
        repo.create(&budget).unwrap();

        let retrieved = repo.get_by_month_category(2026, 3, "Food").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().amount, dec!(600));

        let not_found = repo
            .get_by_month_category(2026, 3, "Transportation")
            .unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_or_create() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);

        // First call creates
        let budget1 = repo.get_or_create(2026, 3, "Food", dec!(600)).unwrap();
        assert_eq!(budget1.amount, dec!(600));

        // Second call returns existing
        let budget2 = repo.get_or_create(2026, 3, "Food", dec!(700)).unwrap();
        assert_eq!(budget2.id, budget1.id);
        assert_eq!(budget2.amount, dec!(600)); // Original amount

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_upsert() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);

        let budget1 = Budget::new(3, 2026, "Food".to_string(), dec!(600));
        repo.upsert(&budget1).unwrap();

        // Upsert with same month/year/category should update
        let mut budget2 = Budget::new(3, 2026, "Food".to_string(), dec!(700));
        budget2.spent = dec!(100);
        repo.upsert(&budget2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].amount, dec!(700));
        assert_eq!(all[0].spent, dec!(100));
    }

    #[test]
    fn test_update_spent() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);

        let budget = create_test_budget();
        repo.create(&budget).unwrap();

        repo.update_spent(&budget.id.to_string(), dec!(250))
            .unwrap();

        let retrieved = repo.get_by_id(&budget.id.to_string()).unwrap().unwrap();
        assert_eq!(retrieved.spent, dec!(250));
    }

    #[test]
    fn test_get_by_year() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);

        let budget1 = Budget::new(1, 2026, "Food".to_string(), dec!(600));
        let budget2 = Budget::new(2, 2026, "Food".to_string(), dec!(600));
        let budget3 = Budget::new(1, 2025, "Food".to_string(), dec!(500));

        repo.create(&budget1).unwrap();
        repo.create(&budget2).unwrap();
        repo.create(&budget3).unwrap();

        let year_2026 = repo.get_by_year(2026).unwrap();
        assert_eq!(year_2026.len(), 2);

        let year_2025 = repo.get_by_year(2025).unwrap();
        assert_eq!(year_2025.len(), 1);
    }

    #[test]
    fn test_copy_from_month() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);

        // Create February budgets
        let budget1 = Budget::new(2, 2026, "Food".to_string(), dec!(600));
        let budget2 = Budget::new(2, 2026, "Transportation".to_string(), dec!(200));
        repo.create(&budget1).unwrap();
        repo.create(&budget2).unwrap();

        // Copy to March
        let copied = repo.copy_from_month(2026, 2, 2026, 3).unwrap();
        assert_eq!(copied.len(), 2);

        let march = repo.get_by_month(2026, 3).unwrap();
        assert_eq!(march.len(), 2);
        assert!(march.iter().all(|b| b.spent == Decimal::ZERO));
    }

    #[test]
    fn test_update() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);
        let mut budget = create_test_budget();

        repo.create(&budget).unwrap();

        budget.amount = dec!(700);
        budget.spent = dec!(350);
        repo.update(&budget).unwrap();

        let retrieved = repo.get_by_id(&budget.id.to_string()).unwrap().unwrap();
        assert_eq!(retrieved.amount, dec!(700));
        assert_eq!(retrieved.spent, dec!(350));
    }

    #[test]
    fn test_delete() {
        let db = create_test_db();
        let repo = BudgetRepository::new(&db);
        let budget = create_test_budget();

        repo.create(&budget).unwrap();
        repo.delete(&budget.id.to_string()).unwrap();

        let retrieved = repo.get_by_id(&budget.id.to_string()).unwrap();
        assert!(retrieved.is_none());
    }
}
