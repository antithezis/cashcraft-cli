//! Income repository
//!
//! Provides CRUD operations for IncomeSource entities using SQLite.

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Row};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use super::{Database, Repository};
use crate::domain::income::{Frequency, IncomeSource};
use crate::error::Result;

/// Repository for managing income sources in the database.
pub struct IncomeRepository<'a> {
    db: &'a Database,
}

impl<'a> IncomeRepository<'a> {
    /// Create a new IncomeRepository with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Get an income source by its variable name.
    ///
    /// # Arguments
    /// * `variable_name` - The variable name to search for (e.g., "salary")
    ///
    /// # Returns
    /// * `Result<Option<IncomeSource>>` - The income source if found
    pub fn get_by_variable_name(&self, variable_name: &str) -> Result<Option<IncomeSource>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, frequency, is_active, 
                    category, start_date, end_date, notes, created_at, updated_at
             FROM income_sources 
             WHERE variable_name = ?1",
        )?;

        let mut rows = stmt.query(params![variable_name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_income_source(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all active income sources.
    ///
    /// # Returns
    /// * `Result<Vec<IncomeSource>>` - All active income sources
    pub fn get_active(&self) -> Result<Vec<IncomeSource>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, frequency, is_active, 
                    category, start_date, end_date, notes, created_at, updated_at
             FROM income_sources 
             WHERE is_active = 1
             ORDER BY display_name",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_income_source(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut sources = Vec::new();
        for row in rows {
            sources.push(row?);
        }

        Ok(sources)
    }

    /// Convert a database row to an IncomeSource.
    fn row_to_income_source(row: &Row<'_>) -> Result<IncomeSource> {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let amount_str: String = row.get(3)?;
        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let frequency_str: String = row.get(4)?;
        let frequency = parse_frequency(&frequency_str)?;

        let is_active: i32 = row.get(5)?;

        let category: Option<String> = row.get(6)?;

        let start_date: Option<String> = row.get(7)?;
        let start_date = start_date
            .map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let end_date: Option<String> = row.get(8)?;
        let end_date = end_date
            .map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let notes: Option<String> = row.get(9)?;

        let created_at_str: String = row.get(10)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Fallback for SQLite CURRENT_TIMESTAMP format
                chrono::NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let updated_at_str: String = row.get(11)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        Ok(IncomeSource {
            id,
            variable_name: row.get(1)?,
            display_name: row.get(2)?,
            amount,
            frequency,
            is_active: is_active != 0,
            category,
            start_date,
            end_date,
            notes,
            created_at,
            updated_at,
        })
    }
}

impl<'a> Repository<IncomeSource> for IncomeRepository<'a> {
    fn create(&self, item: &IncomeSource) -> Result<()> {
        self.db.conn.execute(
            "INSERT INTO income_sources 
             (id, variable_name, display_name, amount, frequency, is_active, 
              category, start_date, end_date, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                item.id.to_string(),
                item.variable_name,
                item.display_name,
                item.amount.to_string(),
                frequency_to_string(&item.frequency),
                if item.is_active { 1 } else { 0 },
                item.category,
                item.start_date.map(|d| d.format("%Y-%m-%d").to_string()),
                item.end_date.map(|d| d.format("%Y-%m-%d").to_string()),
                item.notes,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<IncomeSource>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, frequency, is_active, 
                    category, start_date, end_date, notes, created_at, updated_at
             FROM income_sources 
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_income_source(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_all(&self) -> Result<Vec<IncomeSource>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT id, variable_name, display_name, amount, frequency, is_active, 
                    category, start_date, end_date, notes, created_at, updated_at
             FROM income_sources 
             ORDER BY display_name",
        )?;

        let rows = stmt.query_map([], |row| {
            Self::row_to_income_source(row).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;

        let mut sources = Vec::new();
        for row in rows {
            sources.push(row?);
        }

        Ok(sources)
    }

    fn update(&self, item: &IncomeSource) -> Result<()> {
        self.db.conn.execute(
            "UPDATE income_sources 
             SET variable_name = ?2, display_name = ?3, amount = ?4, frequency = ?5,
                 is_active = ?6, category = ?7, start_date = ?8, end_date = ?9,
                 notes = ?10, updated_at = ?11
             WHERE id = ?1",
            params![
                item.id.to_string(),
                item.variable_name,
                item.display_name,
                item.amount.to_string(),
                frequency_to_string(&item.frequency),
                if item.is_active { 1 } else { 0 },
                item.category,
                item.start_date.map(|d| d.format("%Y-%m-%d").to_string()),
                item.end_date.map(|d| d.format("%Y-%m-%d").to_string()),
                item.notes,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<()> {
        self.db
            .conn
            .execute("DELETE FROM income_sources WHERE id = ?1", params![id])?;
        Ok(())
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

    fn create_test_income() -> IncomeSource {
        IncomeSource::new(
            "salary".to_string(),
            "Primary Job".to_string(),
            dec!(4500),
            Frequency::Monthly,
        )
    }

    #[test]
    fn test_create_and_get_by_id() {
        let db = create_test_db();
        let repo = IncomeRepository::new(&db);
        let income = create_test_income();

        repo.create(&income).unwrap();

        let retrieved = repo.get_by_id(&income.id.to_string()).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.variable_name, "salary");
        assert_eq!(retrieved.display_name, "Primary Job");
        assert_eq!(retrieved.amount, dec!(4500));
    }

    #[test]
    fn test_get_by_variable_name() {
        let db = create_test_db();
        let repo = IncomeRepository::new(&db);
        let income = create_test_income();

        repo.create(&income).unwrap();

        let retrieved = repo.get_by_variable_name("salary").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, income.id);
    }

    #[test]
    fn test_get_all() {
        let db = create_test_db();
        let repo = IncomeRepository::new(&db);

        let income1 = IncomeSource::new(
            "salary".to_string(),
            "Primary Job".to_string(),
            dec!(4500),
            Frequency::Monthly,
        );
        let income2 = IncomeSource::new(
            "freelance".to_string(),
            "Side Projects".to_string(),
            dec!(1200),
            Frequency::Monthly,
        );

        repo.create(&income1).unwrap();
        repo.create(&income2).unwrap();

        let all = repo.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_get_active() {
        let db = create_test_db();
        let repo = IncomeRepository::new(&db);

        let mut income1 = IncomeSource::new(
            "salary".to_string(),
            "Primary Job".to_string(),
            dec!(4500),
            Frequency::Monthly,
        );
        income1.is_active = true;

        let mut income2 = IncomeSource::new(
            "old_job".to_string(),
            "Old Job".to_string(),
            dec!(3000),
            Frequency::Monthly,
        );
        income2.is_active = false;

        repo.create(&income1).unwrap();
        repo.create(&income2).unwrap();

        let active = repo.get_active().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].variable_name, "salary");
    }

    #[test]
    fn test_update() {
        let db = create_test_db();
        let repo = IncomeRepository::new(&db);
        let mut income = create_test_income();

        repo.create(&income).unwrap();

        income.amount = dec!(5000);
        income.display_name = "Updated Job".to_string();
        repo.update(&income).unwrap();

        let retrieved = repo.get_by_id(&income.id.to_string()).unwrap().unwrap();
        assert_eq!(retrieved.amount, dec!(5000));
        assert_eq!(retrieved.display_name, "Updated Job");
    }

    #[test]
    fn test_delete() {
        let db = create_test_db();
        let repo = IncomeRepository::new(&db);
        let income = create_test_income();

        repo.create(&income).unwrap();
        repo.delete(&income.id.to_string()).unwrap();

        let retrieved = repo.get_by_id(&income.id.to_string()).unwrap();
        assert!(retrieved.is_none());
    }
}
