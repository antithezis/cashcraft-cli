//! Database connection and schema management
//!
//! Provides SQLite database initialization and schema setup for CashCraft.

use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;

use crate::error::Result;

/// Database wrapper providing connection management and schema initialization.
pub struct Database {
    /// The SQLite connection
    pub conn: Connection,
}

impl Database {
    /// Open database at the given path.
    ///
    /// Creates the database file if it doesn't exist and initializes the schema.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    ///
    /// # Returns
    /// * `Result<Self>` - The initialized database or an error
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    ///
    /// The database is initialized with the full schema but will be lost
    /// when the connection is closed.
    ///
    /// # Returns
    /// * `Result<Self>` - The initialized in-memory database or an error
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize the database schema.
    ///
    /// Creates all tables and indexes if they don't exist, then runs migrations.
    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA)?;
        self.run_migrations()?;
        Ok(())
    }

    /// Run database migrations for schema updates.
    ///
    /// Applies any necessary schema changes to existing databases.
    fn run_migrations(&self) -> Result<()> {
        // Migration: Add is_template column to budgets table if it doesn't exist
        let has_is_template: bool = self
            .conn
            .prepare("SELECT is_template FROM budgets LIMIT 1")
            .is_ok();

        if !has_is_template {
            // Add the column with default value
            self.conn.execute(
                "ALTER TABLE budgets ADD COLUMN is_template INTEGER DEFAULT 0",
                [],
            )?;
        }

        Ok(())
    }

    /// Begin a transaction.
    ///
    /// Returns a rusqlite Transaction that can be committed or rolled back.
    pub fn transaction(&mut self) -> SqliteResult<rusqlite::Transaction<'_>> {
        self.conn.transaction()
    }
}

/// Database schema SQL.
///
/// Uses TEXT for UUID, Decimal, DateTime, and Date types for SQLite compatibility.
/// All timestamps are stored in ISO 8601 format.
const SCHEMA: &str = r#"
-- Income Sources
CREATE TABLE IF NOT EXISTS income_sources (
    id TEXT PRIMARY KEY,
    variable_name TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    amount TEXT NOT NULL,
    frequency TEXT NOT NULL,
    is_active INTEGER DEFAULT 1,
    category TEXT,
    start_date TEXT,
    end_date TEXT,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Expenses
CREATE TABLE IF NOT EXISTS expenses (
    id TEXT PRIMARY KEY,
    variable_name TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    amount TEXT NOT NULL,
    expense_type TEXT NOT NULL,
    frequency TEXT NOT NULL,
    category TEXT NOT NULL,
    is_active INTEGER DEFAULT 1,
    is_essential INTEGER DEFAULT 0,
    due_day INTEGER,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Transactions
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    amount TEXT NOT NULL,
    transaction_type TEXT NOT NULL,
    category TEXT NOT NULL,
    account TEXT,
    tags TEXT,
    notes TEXT,
    is_recurring INTEGER DEFAULT 0,
    recurring_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Budgets (with template support)
-- is_template=1: Template applies to all months (month/year ignored, set to 0)
-- is_template=0: Override for a specific month/year
CREATE TABLE IF NOT EXISTS budgets (
    id TEXT PRIMARY KEY,
    month INTEGER NOT NULL,
    year INTEGER NOT NULL,
    category TEXT NOT NULL,
    amount TEXT NOT NULL,
    spent TEXT DEFAULT '0',
    is_template INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(month, year, category)
);

-- Playground Sessions
CREATE TABLE IF NOT EXISTS playground_sessions (
    id TEXT PRIMARY KEY,
    name TEXT,
    content TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for faster lookups
CREATE INDEX IF NOT EXISTS idx_income_variable ON income_sources(variable_name);
CREATE INDEX IF NOT EXISTS idx_income_active ON income_sources(is_active);
CREATE INDEX IF NOT EXISTS idx_expense_variable ON expenses(variable_name);
CREATE INDEX IF NOT EXISTS idx_expense_active ON expenses(is_active);
CREATE INDEX IF NOT EXISTS idx_expense_category ON expenses(category);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category);
    CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(transaction_type);
    CREATE INDEX IF NOT EXISTS idx_budgets_month_year ON budgets(month, year);

    -- Monthly Balances
    CREATE TABLE IF NOT EXISTS monthly_balances (
        year INTEGER NOT NULL,
        month INTEGER NOT NULL,
        amount TEXT NOT NULL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (year, month)
    );
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::open_in_memory();
        assert!(db.is_ok());
    }

    #[test]
    fn test_schema_initialization() {
        let db = Database::open_in_memory().unwrap();

        // Verify tables exist by querying sqlite_master
        let mut stmt = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"income_sources".to_string()));
        assert!(tables.contains(&"expenses".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"budgets".to_string()));
        assert!(tables.contains(&"playground_sessions".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn test_indexes_created() {
        let db = Database::open_in_memory().unwrap();

        // Verify indexes exist
        let mut stmt = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap();
        let indexes: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(indexes.contains(&"idx_income_variable".to_string()));
        assert!(indexes.contains(&"idx_expense_variable".to_string()));
        assert!(indexes.contains(&"idx_transactions_date".to_string()));
        assert!(indexes.contains(&"idx_budgets_month_year".to_string()));
    }
}
