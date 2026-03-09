//! Transaction repository
//!
//! Provides CRUD operations for Transaction entities using SQLite.

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Row};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use super::{Database, Repository};
use crate::domain::transaction::{Transaction, TransactionType};
use crate::error::Result;

/// Repository for managing transactions in the database.
pub struct TransactionRepository<'a> {
    db: &'a Database,
}

impl<'a> TransactionRepository<'a> {
    /// Create a new TransactionRepository with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
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
        // Calculate date range for the month
        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| crate::error::CashCraftError::Validation("Invalid date".to_string()))?;

        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or_else(|| crate::error::CashCraftError::Validation("Invalid date".to_string()))?;

        self.get_by_date_range(start_date, end_date.pred_opt().unwrap_or(end_date))
    }

    /// Get transactions by category.
    ///
    /// # Arguments
    /// * `category` - The category to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions in the specified category
    pub fn get_by_category(&self, category: &str) -> Result<Vec<Transaction>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             WHERE category = ?1
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map(params![category], |row| {
            Self::row_to_transaction(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    /// Get transactions within a date range (inclusive).
    ///
    /// # Arguments
    /// * `start` - Start date (inclusive)
    /// * `end` - End date (inclusive)
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions within the date range
    pub fn get_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<Transaction>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map(
            params![
                start.format("%Y-%m-%d").to_string(),
                end.format("%Y-%m-%d").to_string()
            ],
            |row| {
                Self::row_to_transaction(row).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })
            },
        )?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    /// Get transactions by type (Income, Expense, Transfer).
    ///
    /// # Arguments
    /// * `transaction_type` - The type to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions of the specified type
    pub fn get_by_type(&self, transaction_type: &TransactionType) -> Result<Vec<Transaction>> {
        let type_str = transaction_type_to_string(transaction_type);
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             WHERE transaction_type = ?1
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map(params![type_str], |row| {
            Self::row_to_transaction(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    /// Get recurring transactions.
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - All recurring transactions
    pub fn get_recurring(&self) -> Result<Vec<Transaction>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             WHERE is_recurring = 1
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_transaction(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    /// Get transactions by recurring source ID.
    ///
    /// # Arguments
    /// * `recurring_id` - The recurring source ID
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Transactions linked to the recurring source
    pub fn get_by_recurring_id(&self, recurring_id: &str) -> Result<Vec<Transaction>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             WHERE recurring_id = ?1
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map(params![recurring_id], |row| {
            Self::row_to_transaction(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    /// Search transactions by description.
    ///
    /// # Arguments
    /// * `query` - The search query (case-insensitive partial match)
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Matching transactions
    pub fn search(&self, query: &str) -> Result<Vec<Transaction>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             WHERE description LIKE ?1 OR notes LIKE ?1
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map(params![search_pattern], |row| {
            Self::row_to_transaction(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    /// Convert a database row to a Transaction.
    fn row_to_transaction(row: &Row<'_>) -> Result<Transaction> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let date_str: String = row.get(1)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let amount_str: String = row.get(3)?;
        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let transaction_type_str: String = row.get(4)?;
        let transaction_type = parse_transaction_type(&transaction_type_str)?;

        let account: Option<String> = row.get(6)?;

        let tags_str: Option<String> = row.get(7)?;
        let tags: Vec<String> = tags_str
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();

        let notes: Option<String> = row.get(8)?;

        let is_recurring: i32 = row.get(9)?;

        let recurring_id_str: Option<String> = row.get(10)?;
        let recurring_id = recurring_id_str
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

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

        Ok(Transaction {
            id,
            date,
            description: row.get(2)?,
            amount,
            transaction_type,
            category: row.get(5)?,
            account,
            tags,
            notes,
            is_recurring: is_recurring != 0,
            recurring_id,
            created_at,
            updated_at,
        })
    }
}

impl<'a> Repository<Transaction> for TransactionRepository<'a> {
    fn create(&self, item: &Transaction) -> Result<()> {
        let tags_json = serde_json::to_string(&item.tags)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        self.db.conn.execute(
            "INSERT INTO transactions 
             (id, date, description, amount, transaction_type, category,
              account, tags, notes, is_recurring, recurring_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                item.id.to_string(),
                item.date.format("%Y-%m-%d").to_string(),
                item.description,
                item.amount.to_string(),
                transaction_type_to_string(&item.transaction_type),
                item.category,
                item.account,
                tags_json,
                item.notes,
                if item.is_recurring { 1 } else { 0 },
                item.recurring_id.map(|id| id.to_string()),
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<Transaction>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_transaction(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_all(&self) -> Result<Vec<Transaction>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, date, description, amount, transaction_type, category,
                    account, tags, notes, is_recurring, recurring_id, created_at, updated_at
             FROM transactions 
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_transaction(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(row?);
        }

        Ok(transactions)
    }

    fn update(&self, item: &Transaction) -> Result<()> {
        let tags_json = serde_json::to_string(&item.tags)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        self.db.conn.execute(
            "UPDATE transactions 
             SET date = ?2, description = ?3, amount = ?4, transaction_type = ?5,
                 category = ?6, account = ?7, tags = ?8, notes = ?9,
                 is_recurring = ?10, recurring_id = ?11, updated_at = ?12
             WHERE id = ?1",
            params![
                item.id.to_string(),
                item.date.format("%Y-%m-%d").to_string(),
                item.description,
                item.amount.to_string(),
                transaction_type_to_string(&item.transaction_type),
                item.category,
                item.account,
                tags_json,
                item.notes,
                if item.is_recurring { 1 } else { 0 },
                item.recurring_id.map(|id| id.to_string()),
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.db
            .conn
            .execute("DELETE FROM transactions WHERE id = ?1", params![id])?;
        Ok(())
    }
}

/// Convert TransactionType enum to string for storage.
fn transaction_type_to_string(tt: &TransactionType) -> &'static str {
    match tt {
        TransactionType::Income => "Income",
        TransactionType::Expense => "Expense",
        TransactionType::Transfer => "Transfer",
    }
}

/// Parse TransactionType from string.
fn parse_transaction_type(s: &str) -> Result<TransactionType> {
    match s {
        "Income" => Ok(TransactionType::Income),
        "Expense" => Ok(TransactionType::Expense),
        "Transfer" => Ok(TransactionType::Transfer),
        _ => Err(crate::error::CashCraftError::Parse(format!(
            "Unknown transaction type: {}",
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
    fn test_create_and_get_by_id() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);
        let tx = create_test_transaction();

        repo.create(&tx).unwrap();

        let retrieved = repo.get_by_id(&tx.id.to_string()).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.description, "Grocery Shopping");
        assert_eq!(retrieved.amount, dec!(127.45));
        assert_eq!(retrieved.category, "Food");
    }

    #[test]
    fn test_get_by_month() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap(),
            "March expense".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 2, 15).unwrap(),
            "February expense".to_string(),
            dec!(200),
            TransactionType::Expense,
            "Food".to_string(),
        );

        repo.create(&tx1).unwrap();
        repo.create(&tx2).unwrap();

        let march = repo.get_by_month(2026, 3).unwrap();
        assert_eq!(march.len(), 1);
        assert_eq!(march[0].description, "March expense");

        let feb = repo.get_by_month(2026, 2).unwrap();
        assert_eq!(feb.len(), 1);
        assert_eq!(feb[0].description, "February expense");
    }

    #[test]
    fn test_get_by_category() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap(),
            "Groceries".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 8).unwrap(),
            "Gas".to_string(),
            dec!(50),
            TransactionType::Expense,
            "Transportation".to_string(),
        );

        repo.create(&tx1).unwrap();
        repo.create(&tx2).unwrap();

        let food = repo.get_by_category("Food").unwrap();
        assert_eq!(food.len(), 1);
        assert_eq!(food[0].description, "Groceries");
    }

    #[test]
    fn test_get_by_date_range() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            "Early March".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            "Mid March".to_string(),
            dec!(200),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx3 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
            "Late March".to_string(),
            dec!(300),
            TransactionType::Expense,
            "Food".to_string(),
        );

        repo.create(&tx1).unwrap();
        repo.create(&tx2).unwrap();
        repo.create(&tx3).unwrap();

        let range = repo
            .get_by_date_range(
                NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
                NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
            )
            .unwrap();

        assert_eq!(range.len(), 1);
        assert_eq!(range[0].description, "Mid March");
    }

    #[test]
    fn test_get_by_type() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap(),
            "Salary".to_string(),
            dec!(4500),
            TransactionType::Income,
            "Salary".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap(),
            "Groceries".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );

        repo.create(&tx1).unwrap();
        repo.create(&tx2).unwrap();

        let income = repo.get_by_type(&TransactionType::Income).unwrap();
        assert_eq!(income.len(), 1);
        assert_eq!(income[0].description, "Salary");

        let expenses = repo.get_by_type(&TransactionType::Expense).unwrap();
        assert_eq!(expenses.len(), 1);
        assert_eq!(expenses[0].description, "Groceries");
    }

    #[test]
    fn test_search() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 9).unwrap(),
            "Grocery Store".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Food".to_string(),
        );
        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 8).unwrap(),
            "Gas Station".to_string(),
            dec!(50),
            TransactionType::Expense,
            "Transportation".to_string(),
        );

        repo.create(&tx1).unwrap();
        repo.create(&tx2).unwrap();

        let results = repo.search("grocery").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].description, "Grocery Store");
    }

    #[test]
    fn test_update() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);
        let mut tx = create_test_transaction();

        repo.create(&tx).unwrap();

        tx.amount = dec!(150);
        tx.description = "Updated description".to_string();
        repo.update(&tx).unwrap();

        let retrieved = repo.get_by_id(&tx.id.to_string()).unwrap().unwrap();
        assert_eq!(retrieved.amount, dec!(150));
        assert_eq!(retrieved.description, "Updated description");
    }

    #[test]
    fn test_delete() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);
        let tx = create_test_transaction();

        repo.create(&tx).unwrap();
        repo.delete(&tx.id.to_string()).unwrap();

        let retrieved = repo.get_by_id(&tx.id.to_string()).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_tags_serialization() {
        let db = create_test_db();
        let repo = TransactionRepository::new(&db);
        let mut tx = create_test_transaction();
        tx.tags = vec!["groceries".to_string(), "essential".to_string()];

        repo.create(&tx).unwrap();

        let retrieved = repo.get_by_id(&tx.id.to_string()).unwrap().unwrap();
        assert_eq!(retrieved.tags.len(), 2);
        assert!(retrieved.tags.contains(&"groceries".to_string()));
        assert!(retrieved.tags.contains(&"essential".to_string()));
    }
}
