use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use rust_decimal::Decimal;
use std::str::FromStr;

use super::Database;
use crate::domain::balance::MonthlyBalance;
use crate::error::Result;

pub struct BalanceRepository<'a> {
    db: &'a Database,
}

impl<'a> BalanceRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn get(&self, year: i32, month: u32) -> Result<Option<MonthlyBalance>> {
        let mut stmt = self.db.conn.prepare(
            "SELECT year, month, amount, created_at, updated_at
             FROM monthly_balances
             WHERE year = ?1 AND month = ?2",
        )?;

        let mut rows = stmt.query(params![year, month as i32])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_balance(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn set(&self, year: i32, month: u32, amount: Decimal) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db.conn.execute(
            "INSERT INTO monthly_balances (year, month, amount, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)
             ON CONFLICT(year, month) DO UPDATE SET
                 amount = excluded.amount,
                 updated_at = excluded.updated_at",
            params![year, month as i32, amount.to_string(), now],
        )?;
        Ok(())
    }

    fn row_to_balance(row: &Row<'_>) -> Result<MonthlyBalance> {
        let year: i32 = row.get(0)?;
        let month: i32 = row.get(1)?;
        let amount_str: String = row.get(2)?;
        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let created_at_str: String = row.get(3)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        let updated_at_str: String = row.get(4)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .map_err(|e| crate::error::CashCraftError::Parse(e.to_string()))?;

        Ok(MonthlyBalance {
            year,
            month: month as u32,
            amount,
            created_at,
            updated_at,
        })
    }
}
