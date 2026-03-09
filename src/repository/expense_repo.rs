//! Expense repository
//!
//! Provides CRUD operations for Expense entities using SQLite.

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use super::{Database, Repository};
use crate::domain::expense::{Expense, ExpenseCategory, ExpenseType};
use crate::domain::income::Frequency;
use crate::error::Result;

/// Repository for managing expenses in the database.
pub struct ExpenseRepository<'a> {
    db: &'a Database,
}

impl<'a> ExpenseRepository<'a> {
    /// Create a new ExpenseRepository with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Get an expense by its variable name.
    ///
    /// # Arguments
    /// * `variable_name` - The variable name to search for (e.g., "rent")
    ///
    /// # Returns
    /// * `Result<Option<Expense>>` - The expense if found
    pub fn get_by_variable_name(&self, variable_name: &str) -> Result<Option<Expense>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, expense_type, frequency,
                    category, is_active, is_essential, due_day, notes, created_at, updated_at
             FROM expenses 
             WHERE variable_name = ?1",
        )?;

        let mut rows = stmt.query(params![variable_name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_expense(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all active expenses.
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - All active expenses
    pub fn get_active(&self) -> Result<Vec<Expense>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, expense_type, frequency,
                    category, is_active, is_essential, due_day, notes, created_at, updated_at
             FROM expenses 
             WHERE is_active = 1
             ORDER BY display_name",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_expense(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(row?);
        }

        Ok(expenses)
    }

    /// Get expenses by category.
    ///
    /// # Arguments
    /// * `category` - The category to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - Expenses in the specified category
    pub fn get_by_category(&self, category: &str) -> Result<Vec<Expense>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, expense_type, frequency,
                    category, is_active, is_essential, due_day, notes, created_at, updated_at
             FROM expenses 
             WHERE category = ?1
             ORDER BY display_name",
        )?;

        let rows = stmt.query_map(params![category], |row| {
            Self::row_to_expense(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(row?);
        }

        Ok(expenses)
    }

    /// Get expenses by type (Fixed, Variable, OneTime).
    ///
    /// # Arguments
    /// * `expense_type` - The type to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - Expenses of the specified type
    pub fn get_by_type(&self, expense_type: &ExpenseType) -> Result<Vec<Expense>> {
        let type_str = expense_type_to_string(expense_type);
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, expense_type, frequency,
                    category, is_active, is_essential, due_day, notes, created_at, updated_at
             FROM expenses 
             WHERE expense_type = ?1
             ORDER BY display_name",
        )?;

        let rows = stmt.query_map(params![type_str], |row| {
            Self::row_to_expense(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(row?);
        }

        Ok(expenses)
    }

    /// Get essential expenses only.
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - All essential expenses
    pub fn get_essential(&self) -> Result<Vec<Expense>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, expense_type, frequency,
                    category, is_active, is_essential, due_day, notes, created_at, updated_at
             FROM expenses 
             WHERE is_essential = 1 AND is_active = 1
             ORDER BY display_name",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_expense(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(row?);
        }

        Ok(expenses)
    }

    /// Convert a database row to an Expense.
    fn row_to_expense(row: &Row<'_>) -> Result<Expense> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let amount_str: String = row.get(3)?;
        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let expense_type_str: String = row.get(4)?;
        let expense_type = parse_expense_type(&expense_type_str)?;

        let frequency_str: String = row.get(5)?;
        let frequency = parse_frequency(&frequency_str)?;

        let category_str: String = row.get(6)?;
        let category = parse_expense_category(&category_str);

        let is_active: i32 = row.get(7)?;
        let is_essential: i32 = row.get(8)?;
        let due_day: Option<i32> = row.get(9)?;
        let notes: Option<String> = row.get(10)?;

        let created_at_str: String = row.get(11)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let updated_at_str: String = row.get(12)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        Ok(Expense {
            id,
            variable_name: row.get(1)?,
            display_name: row.get(2)?,
            amount,
            expense_type,
            frequency,
            category,
            is_active: is_active != 0,
            is_essential: is_essential != 0,
            due_day: due_day.map(|d| d as u8),
            notes,
            created_at,
            updated_at,
        })
    }
}

impl<'a> Repository<Expense> for ExpenseRepository<'a> {
    fn create(&self, item: &Expense) -> Result<()> {
        self.db.conn.execute(
            "INSERT INTO expenses 
             (id, variable_name, display_name, amount, expense_type, frequency,
              category, is_active, is_essential, due_day, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                item.id.to_string(),
                item.variable_name,
                item.display_name,
                item.amount.to_string(),
                expense_type_to_string(&item.expense_type),
                frequency_to_string(&item.frequency),
                item.category.as_str(),
                if item.is_active { 1 } else { 0 },
                if item.is_essential { 1 } else { 0 },
                item.due_day.map(|d| d as i32),
                item.notes,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Expense>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, expense_type, frequency,
                    category, is_active, is_essential, due_day, notes, created_at, updated_at
             FROM expenses 
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_expense(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_all(&self) -> Result<Vec<Expense>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, expense_type, frequency,
                    category, is_active, is_essential, due_day, notes, created_at, updated_at
             FROM expenses 
             ORDER BY display_name",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_expense(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut expenses = Vec::new();
        for row in rows {
            expenses.push(row?);
        }

        Ok(expenses)
    }

    fn update(&self, item: &Expense) -> Result<()> {
        self.db.conn.execute(
            "UPDATE expenses 
             SET variable_name = ?2, display_name = ?3, amount = ?4, expense_type = ?5,
                 frequency = ?6, category = ?7, is_active = ?8, is_essential = ?9,
                 due_day = ?10, notes = ?11, updated_at = ?12
             WHERE id = ?1",
            params![
                item.id.to_string(),
                item.variable_name,
                item.display_name,
                item.amount.to_string(),
                expense_type_to_string(&item.expense_type),
                frequency_to_string(&item.frequency),
                item.category.as_str(),
                if item.is_active { 1 } else { 0 },
                if item.is_essential { 1 } else { 0 },
                item.due_day.map(|d| d as i32),
                item.notes,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.db
            .conn
            .execute("DELETE FROM expenses WHERE id = ?1", params![id])?;
        Ok(())
    }
}

/// Convert ExpenseType enum to string for storage.
fn expense_type_to_string(et: &ExpenseType) -> &'static str {
    match et {
        ExpenseType::Fixed => "Fixed",
        ExpenseType::Variable => "Variable",
        ExpenseType::OneTime => "OneTime",
    }
}

/// Parse ExpenseType from string.
fn parse_expense_type(s: &str) -> Result<ExpenseType> {
    match s {
        "Fixed" => Ok(ExpenseType::Fixed),
        "Variable" => Ok(ExpenseType::Variable),
        "OneTime" => Ok(ExpenseType::OneTime),
        _ => Err(crate::error::CashCraftError::Parse(format!(
            "Unknown expense type: {}",
            s
        ))),
    }
}

/// Parse ExpenseCategory from string.
fn parse_expense_category(s: &str) -> ExpenseCategory {
    match s {
        "Housing" => ExpenseCategory::Housing,
        "Transportation" => ExpenseCategory::Transportation,
        "Food" => ExpenseCategory::Food,
        "Healthcare" => ExpenseCategory::Healthcare,
        "Entertainment" => ExpenseCategory::Entertainment,
        "Utilities" => ExpenseCategory::Utilities,
        "Insurance" => ExpenseCategory::Insurance,
        "Subscriptions" => ExpenseCategory::Subscriptions,
        "PersonalCare" => ExpenseCategory::PersonalCare,
        "Education" => ExpenseCategory::Education,
        "Savings" => ExpenseCategory::Savings,
        "Debt" => ExpenseCategory::Debt,
        _ => ExpenseCategory::Custom(s.to_string()),
    }
}

/// Convert Frequency enum to string for storage.
fn frequency_to_string(freq: &Frequency) -> &'static str {
    match freq {
        Frequency::Daily => "Daily",
        Frequency::Weekly => "Weekly",
        Frequency::BiWeekly => "BiWeekly",
        Frequency::Monthly => "Monthly",
        Frequency::Quarterly => "Quarterly",
        Frequency::Yearly => "Yearly",
        Frequency::OneTime => "OneTime",
    }
}

/// Parse Frequency from string.
fn parse_frequency(s: &str) -> Result<Frequency> {
    match s {
        "Daily" => Ok(Frequency::Daily),
        "Weekly" => Ok(Frequency::Weekly),
        "BiWeekly" => Ok(Frequency::BiWeekly),
        "Monthly" => Ok(Frequency::Monthly),
        "Quarterly" => Ok(Frequency::Quarterly),
        "Yearly" => Ok(Frequency::Yearly),
        "OneTime" => Ok(Frequency::OneTime),
        _ => Err(crate::error::CashCraftError::Parse(format!(
            "Unknown frequency: {}",
            s
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn create_test_expense() -> Expense {
        Expense::new(
            "rent".to_string(),
            "Apartment Rent".to_string(),
            dec!(1500),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        )
    }

    #[test]
    fn test_create_and_get_by_id() {
        let db = create_test_db();
        let repo = ExpenseRepository::new(&db);
        let expense = create_test_expense();

        repo.create(&expense).unwrap();

        let retrieved = repo.get_by_id(&expense.id.to_string()).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.variable_name, "rent");
        assert_eq!(retrieved.display_name, "Apartment Rent");
        assert_eq!(retrieved.amount, dec!(1500));
    }

    #[test]
    fn test_get_by_variable_name() {
        let db = create_test_db();
        let repo = ExpenseRepository::new(&db);
        let expense = create_test_expense();

        repo.create(&expense).unwrap();

        let retrieved = repo.get_by_variable_name("rent").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, expense.id);
    }

    #[test]
    fn test_get_by_category() {
        let db = create_test_db();
        let repo = ExpenseRepository::new(&db);

        let expense1 = Expense::new(
            "rent".to_string(),
            "Apartment".to_string(),
            dec!(1500),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        );
        let expense2 = Expense::new(
            "groceries".to_string(),
            "Food".to_string(),
            dec!(600),
            ExpenseType::Variable,
            Frequency::Monthly,
            ExpenseCategory::Food,
        );

        repo.create(&expense1).unwrap();
        repo.create(&expense2).unwrap();

        let housing = repo.get_by_category("Housing").unwrap();
        assert_eq!(housing.len(), 1);
        assert_eq!(housing[0].variable_name, "rent");
    }

    #[test]
    fn test_get_by_type() {
        let db = create_test_db();
        let repo = ExpenseRepository::new(&db);

        let expense1 = Expense::new(
            "rent".to_string(),
            "Apartment".to_string(),
            dec!(1500),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        );
        let expense2 = Expense::new(
            "groceries".to_string(),
            "Food".to_string(),
            dec!(600),
            ExpenseType::Variable,
            Frequency::Monthly,
            ExpenseCategory::Food,
        );

        repo.create(&expense1).unwrap();
        repo.create(&expense2).unwrap();

        let fixed = repo.get_by_type(&ExpenseType::Fixed).unwrap();
        assert_eq!(fixed.len(), 1);
        assert_eq!(fixed[0].variable_name, "rent");

        let variable = repo.get_by_type(&ExpenseType::Variable).unwrap();
        assert_eq!(variable.len(), 1);
        assert_eq!(variable[0].variable_name, "groceries");
    }

    #[test]
    fn test_get_active() {
        let db = create_test_db();
        let repo = ExpenseRepository::new(&db);

        let mut expense1 = create_test_expense();
        expense1.is_active = true;

        let mut expense2 = Expense::new(
            "old_sub".to_string(),
            "Old Subscription".to_string(),
            dec!(50),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Subscriptions,
        );
        expense2.is_active = false;

        repo.create(&expense1).unwrap();
        repo.create(&expense2).unwrap();

        let active = repo.get_active().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].variable_name, "rent");
    }

    #[test]
    fn test_update() {
        let db = create_test_db();
        let repo = ExpenseRepository::new(&db);
        let mut expense = create_test_expense();

        repo.create(&expense).unwrap();

        expense.amount = dec!(1600);
        expense.is_essential = true;
        repo.update(&expense).unwrap();

        let retrieved = repo.get_by_id(&expense.id.to_string()).unwrap().unwrap();
        assert_eq!(retrieved.amount, dec!(1600));
        assert!(retrieved.is_essential);
    }

    #[test]
    fn test_delete() {
        let db = create_test_db();
        let repo = ExpenseRepository::new(&db);
        let expense = create_test_expense();

        repo.create(&expense).unwrap();
        repo.delete(&expense.id.to_string()).unwrap();

        let retrieved = repo.get_by_id(&expense.id.to_string()).unwrap();
        assert!(retrieved.is_none());
    }
}
