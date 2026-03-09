//! Export service
//!
//! Business logic for exporting and importing financial data.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

use crate::domain::budget::Budget;
use crate::domain::expense::{Expense, ExpenseCategory, ExpenseType};
use crate::domain::income::{Frequency, IncomeSource};
use crate::domain::transaction::{Transaction, TransactionType};
use crate::error::{CashCraftError, Result};
use crate::repository::Database;
use crate::services::{BudgetService, ExpenseService, IncomeService, TransactionService};

/// Exported data structure containing all financial data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedData {
    /// Export version for compatibility
    pub version: String,
    /// Export timestamp
    pub exported_at: String,
    /// Income sources
    pub income_sources: Vec<ExportedIncome>,
    /// Expenses
    pub expenses: Vec<ExportedExpense>,
    /// Transactions
    pub transactions: Vec<ExportedTransaction>,
    /// Budgets
    pub budgets: Vec<ExportedBudget>,
}

/// Exported income source (serialization-friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedIncome {
    pub id: String,
    pub variable_name: String,
    pub display_name: String,
    pub amount: String,
    pub frequency: String,
    pub is_active: bool,
    pub category: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
}

/// Exported expense (serialization-friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedExpense {
    pub id: String,
    pub variable_name: String,
    pub display_name: String,
    pub amount: String,
    pub expense_type: String,
    pub frequency: String,
    pub category: String,
    pub is_active: bool,
    pub is_essential: bool,
    pub due_day: Option<u8>,
    pub notes: Option<String>,
}

/// Exported transaction (serialization-friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedTransaction {
    pub id: String,
    pub date: String,
    pub description: String,
    pub amount: String,
    pub transaction_type: String,
    pub category: String,
    pub account: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub is_recurring: bool,
}

/// Exported budget (serialization-friendly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedBudget {
    pub id: String,
    pub month: u32,
    pub year: i32,
    pub category: String,
    pub amount: String,
    pub spent: String,
}

/// CSV row for transaction export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCsvRow {
    pub date: String,
    pub description: String,
    pub amount: String,
    pub transaction_type: String,
    pub category: String,
    pub account: String,
    pub tags: String,
    pub notes: String,
}

/// Service for exporting and importing data.
pub struct ExportService<'a> {
    income_service: IncomeService<'a>,
    expense_service: ExpenseService<'a>,
    transaction_service: TransactionService<'a>,
    budget_service: BudgetService<'a>,
}

impl<'a> ExportService<'a> {
    /// Create a new ExportService with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self {
            income_service: IncomeService::new(db),
            expense_service: ExpenseService::new(db),
            transaction_service: TransactionService::new(db),
            budget_service: BudgetService::new(db),
        }
    }

    /// Export transactions to CSV format.
    ///
    /// # Arguments
    /// * `transactions` - The transactions to export
    /// * `path` - The file path to write to
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn export_csv<P: AsRef<Path>>(&self, transactions: &[Transaction], path: P) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write CSV header
        writeln!(
            writer,
            "date,description,amount,type,category,account,tags,notes"
        )?;

        // Write transaction rows
        for tx in transactions {
            let row = TransactionCsvRow {
                date: tx.date.format("%Y-%m-%d").to_string(),
                description: escape_csv(&tx.description),
                amount: tx.amount.to_string(),
                transaction_type: match tx.transaction_type {
                    TransactionType::Income => "Income".to_string(),
                    TransactionType::Expense => "Expense".to_string(),
                    TransactionType::Transfer => "Transfer".to_string(),
                },
                category: escape_csv(&tx.category),
                account: tx.account.clone().unwrap_or_default(),
                tags: tx.tags.join(";"),
                notes: tx.notes.clone().unwrap_or_default(),
            };

            writeln!(
                writer,
                "{},{},{},{},{},{},{},{}",
                row.date,
                row.description,
                row.amount,
                row.transaction_type,
                row.category,
                escape_csv(&row.account),
                escape_csv(&row.tags),
                escape_csv(&row.notes)
            )?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Export all data to JSON format.
    ///
    /// # Arguments
    /// * `path` - The file path to write to
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn export_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = self.collect_export_data()?;
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &data)
            .map_err(|e| CashCraftError::Parse(e.to_string()))?;
        Ok(())
    }

    /// Import transactions from CSV.
    ///
    /// # Arguments
    /// * `path` - The file path to read from
    ///
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Imported transactions (not yet saved)
    pub fn import_csv<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Transaction>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut transactions = Vec::new();

        for result in csv_reader.records() {
            let record: csv::StringRecord =
                result.map_err(|e: csv::Error| CashCraftError::Parse(e.to_string()))?;

            if record.len() < 5 {
                continue; // Skip malformed rows
            }

            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")
                .map_err(|e| CashCraftError::Parse(e.to_string()))?;

            let description = record[1].to_string();

            let amount =
                Decimal::from_str(&record[2]).map_err(|e| CashCraftError::Parse(e.to_string()))?;

            let transaction_type = match record[3].to_lowercase().as_str() {
                "income" => TransactionType::Income,
                "expense" => TransactionType::Expense,
                "transfer" => TransactionType::Transfer,
                _ => TransactionType::Expense,
            };

            let category = record[4].to_string();

            let mut tx = Transaction::new(date, description, amount, transaction_type, category);

            // Optional fields
            if record.len() > 5 && !record[5].is_empty() {
                tx.account = Some(record[5].to_string());
            }
            if record.len() > 6 && !record[6].is_empty() {
                tx.tags = record[6].split(';').map(|s: &str| s.to_string()).collect();
            }
            if record.len() > 7 && !record[7].is_empty() {
                tx.notes = Some(record[7].to_string());
            }

            transactions.push(tx);
        }

        Ok(transactions)
    }

    /// Import all data from JSON.
    ///
    /// # Arguments
    /// * `path` - The file path to read from
    ///
    /// # Returns
    /// * `Result<ExportedData>` - Imported data (not yet saved)
    pub fn import_json<P: AsRef<Path>>(&self, path: P) -> Result<ExportedData> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: ExportedData =
            serde_json::from_reader(reader).map_err(|e| CashCraftError::Parse(e.to_string()))?;
        Ok(data)
    }

    /// Export transactions for a date range to CSV.
    ///
    /// # Arguments
    /// * `start` - Start date (inclusive)
    /// * `end` - End date (inclusive)
    /// * `path` - The file path to write to
    ///
    /// # Returns
    /// * `Result<usize>` - Number of transactions exported
    pub fn export_transactions_range<P: AsRef<Path>>(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        path: P,
    ) -> Result<usize> {
        let transactions = self.transaction_service.get_by_date_range(start, end)?;
        let count = transactions.len();
        self.export_csv(&transactions, path)?;
        Ok(count)
    }

    /// Export transactions by category to CSV.
    ///
    /// # Arguments
    /// * `category` - The category to filter by
    /// * `path` - The file path to write to
    ///
    /// # Returns
    /// * `Result<usize>` - Number of transactions exported
    pub fn export_transactions_category<P: AsRef<Path>>(
        &self,
        category: &str,
        path: P,
    ) -> Result<usize> {
        let transactions = self.transaction_service.get_by_category(category)?;
        let count = transactions.len();
        self.export_csv(&transactions, path)?;
        Ok(count)
    }

    /// Collect all data for export.
    fn collect_export_data(&self) -> Result<ExportedData> {
        let income_sources = self.income_service.get_all()?;
        let expenses = self.expense_service.get_all()?;
        let transactions = self.transaction_service.get_all()?;
        let budgets = self.budget_service.get_all()?;

        Ok(ExportedData {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            income_sources: income_sources.into_iter().map(|i| i.into()).collect(),
            expenses: expenses.into_iter().map(|e| e.into()).collect(),
            transactions: transactions.into_iter().map(|t| t.into()).collect(),
            budgets: budgets.into_iter().map(|b| b.into()).collect(),
        })
    }
}

impl From<IncomeSource> for ExportedIncome {
    fn from(income: IncomeSource) -> Self {
        Self {
            id: income.id.to_string(),
            variable_name: income.variable_name,
            display_name: income.display_name,
            amount: income.amount.to_string(),
            frequency: match income.frequency {
                Frequency::Daily => "Daily".to_string(),
                Frequency::Weekly => "Weekly".to_string(),
                Frequency::BiWeekly => "BiWeekly".to_string(),
                Frequency::Monthly => "Monthly".to_string(),
                Frequency::Quarterly => "Quarterly".to_string(),
                Frequency::Yearly => "Yearly".to_string(),
                Frequency::OneTime => "OneTime".to_string(),
            },
            is_active: income.is_active,
            category: income.category,
            start_date: income.start_date.map(|d| d.format("%Y-%m-%d").to_string()),
            end_date: income.end_date.map(|d| d.format("%Y-%m-%d").to_string()),
            notes: income.notes,
        }
    }
}

impl From<Expense> for ExportedExpense {
    fn from(expense: Expense) -> Self {
        Self {
            id: expense.id.to_string(),
            variable_name: expense.variable_name,
            display_name: expense.display_name,
            amount: expense.amount.to_string(),
            expense_type: match expense.expense_type {
                ExpenseType::Fixed => "Fixed".to_string(),
                ExpenseType::Variable => "Variable".to_string(),
                ExpenseType::OneTime => "OneTime".to_string(),
            },
            frequency: match expense.frequency {
                Frequency::Daily => "Daily".to_string(),
                Frequency::Weekly => "Weekly".to_string(),
                Frequency::BiWeekly => "BiWeekly".to_string(),
                Frequency::Monthly => "Monthly".to_string(),
                Frequency::Quarterly => "Quarterly".to_string(),
                Frequency::Yearly => "Yearly".to_string(),
                Frequency::OneTime => "OneTime".to_string(),
            },
            category: expense.category.as_str().to_string(),
            is_active: expense.is_active,
            is_essential: expense.is_essential,
            due_day: expense.due_day,
            notes: expense.notes,
        }
    }
}

impl From<Transaction> for ExportedTransaction {
    fn from(tx: Transaction) -> Self {
        Self {
            id: tx.id.to_string(),
            date: tx.date.format("%Y-%m-%d").to_string(),
            description: tx.description,
            amount: tx.amount.to_string(),
            transaction_type: match tx.transaction_type {
                TransactionType::Income => "Income".to_string(),
                TransactionType::Expense => "Expense".to_string(),
                TransactionType::Transfer => "Transfer".to_string(),
            },
            category: tx.category,
            account: tx.account,
            tags: tx.tags,
            notes: tx.notes,
            is_recurring: tx.is_recurring,
        }
    }
}

impl From<Budget> for ExportedBudget {
    fn from(budget: Budget) -> Self {
        Self {
            id: budget.id.to_string(),
            month: budget.month,
            year: budget.year,
            category: budget.category,
            amount: budget.amount.to_string(),
            spent: budget.spent.to_string(),
        }
    }
}

/// Escape a string for CSV output.
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Convert exported income back to domain type.
pub fn exported_to_income(exported: &ExportedIncome) -> Result<IncomeSource> {
    let frequency = match exported.frequency.as_str() {
        "Daily" => Frequency::Daily,
        "Weekly" => Frequency::Weekly,
        "BiWeekly" => Frequency::BiWeekly,
        "Monthly" => Frequency::Monthly,
        "Quarterly" => Frequency::Quarterly,
        "Yearly" => Frequency::Yearly,
        "OneTime" => Frequency::OneTime,
        _ => {
            return Err(CashCraftError::Parse(format!(
                "Unknown frequency: {}",
                exported.frequency
            )))
        }
    };

    let amount =
        Decimal::from_str(&exported.amount).map_err(|e| CashCraftError::Parse(e.to_string()))?;

    let mut income = IncomeSource::new(
        exported.variable_name.clone(),
        exported.display_name.clone(),
        amount,
        frequency,
    );

    income.is_active = exported.is_active;
    income.category = exported.category.clone();
    income.notes = exported.notes.clone();

    if let Some(ref date_str) = exported.start_date {
        income.start_date = Some(
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| CashCraftError::Parse(e.to_string()))?,
        );
    }

    if let Some(ref date_str) = exported.end_date {
        income.end_date = Some(
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| CashCraftError::Parse(e.to_string()))?,
        );
    }

    Ok(income)
}

/// Convert exported expense back to domain type.
pub fn exported_to_expense(exported: &ExportedExpense) -> Result<Expense> {
    let expense_type = match exported.expense_type.as_str() {
        "Fixed" => ExpenseType::Fixed,
        "Variable" => ExpenseType::Variable,
        "OneTime" => ExpenseType::OneTime,
        _ => {
            return Err(CashCraftError::Parse(format!(
                "Unknown expense type: {}",
                exported.expense_type
            )))
        }
    };

    let frequency = match exported.frequency.as_str() {
        "Daily" => Frequency::Daily,
        "Weekly" => Frequency::Weekly,
        "BiWeekly" => Frequency::BiWeekly,
        "Monthly" => Frequency::Monthly,
        "Quarterly" => Frequency::Quarterly,
        "Yearly" => Frequency::Yearly,
        "OneTime" => Frequency::OneTime,
        _ => {
            return Err(CashCraftError::Parse(format!(
                "Unknown frequency: {}",
                exported.frequency
            )))
        }
    };

    let category = match exported.category.as_str() {
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
        other => ExpenseCategory::Custom(other.to_string()),
    };

    let amount =
        Decimal::from_str(&exported.amount).map_err(|e| CashCraftError::Parse(e.to_string()))?;

    let mut expense = Expense::new(
        exported.variable_name.clone(),
        exported.display_name.clone(),
        amount,
        expense_type,
        frequency,
        category,
    );

    expense.is_active = exported.is_active;
    expense.is_essential = exported.is_essential;
    expense.due_day = exported.due_day;
    expense.notes = exported.notes.clone();

    Ok(expense)
}

/// Convert exported transaction back to domain type.
pub fn exported_to_transaction(exported: &ExportedTransaction) -> Result<Transaction> {
    let transaction_type = match exported.transaction_type.as_str() {
        "Income" => TransactionType::Income,
        "Expense" => TransactionType::Expense,
        "Transfer" => TransactionType::Transfer,
        _ => {
            return Err(CashCraftError::Parse(format!(
                "Unknown transaction type: {}",
                exported.transaction_type
            )))
        }
    };

    let date = NaiveDate::parse_from_str(&exported.date, "%Y-%m-%d")
        .map_err(|e| CashCraftError::Parse(e.to_string()))?;

    let amount =
        Decimal::from_str(&exported.amount).map_err(|e| CashCraftError::Parse(e.to_string()))?;

    let mut tx = Transaction::new(
        date,
        exported.description.clone(),
        amount,
        transaction_type,
        exported.category.clone(),
    );

    tx.account = exported.account.clone();
    tx.tags = exported.tags.clone();
    tx.notes = exported.notes.clone();
    tx.is_recurring = exported.is_recurring;

    Ok(tx)
}

/// Convert exported budget back to domain type.
pub fn exported_to_budget(exported: &ExportedBudget) -> Result<Budget> {
    let amount =
        Decimal::from_str(&exported.amount).map_err(|e| CashCraftError::Parse(e.to_string()))?;

    let spent =
        Decimal::from_str(&exported.spent).map_err(|e| CashCraftError::Parse(e.to_string()))?;

    let mut budget = Budget::new(
        exported.month,
        exported.year,
        exported.category.clone(),
        amount,
    );

    budget.spent = spent;

    Ok(budget)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::io::Read;
    use tempfile::tempdir;

    fn create_test_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_export_csv() {
        let db = create_test_db();
        let service = ExportService::new(&db);

        let transactions = vec![
            Transaction::new(
                NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                "Salary".to_string(),
                dec!(4500),
                TransactionType::Income,
                "Salary".to_string(),
            ),
            Transaction::new(
                NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
                "Rent".to_string(),
                dec!(1500),
                TransactionType::Expense,
                "Housing".to_string(),
            ),
        ];

        let dir = tempdir().unwrap();
        let path = dir.path().join("export.csv");

        service.export_csv(&transactions, &path).unwrap();

        let mut content = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("date,description,amount"));
        assert!(content.contains("2026-03-01"));
        assert!(content.contains("Salary"));
        assert!(content.contains("4500"));
    }

    #[test]
    fn test_export_import_json() {
        let db = create_test_db();

        // Add some data
        let income_service = IncomeService::new(&db);
        let income = IncomeSource::new(
            "salary".to_string(),
            "Primary Job".to_string(),
            dec!(4500),
            Frequency::Monthly,
        );
        income_service.create(&income).unwrap();

        let export_service = ExportService::new(&db);

        let dir = tempdir().unwrap();
        let path = dir.path().join("export.json");

        // Export
        export_service.export_json(&path).unwrap();

        // Import
        let imported = export_service.import_json(&path).unwrap();

        assert_eq!(imported.version, "1.0");
        assert_eq!(imported.income_sources.len(), 1);
        assert_eq!(imported.income_sources[0].variable_name, "salary");
    }

    #[test]
    fn test_import_csv() {
        let db = create_test_db();
        let service = ExportService::new(&db);

        let dir = tempdir().unwrap();
        let path = dir.path().join("import.csv");

        // Create a CSV file
        let csv_content = "date,description,amount,type,category,account,tags,notes\n\
                          2026-03-01,Salary,4500,Income,Salary,Bank,paycheck,Monthly salary\n\
                          2026-03-05,Rent,1500,Expense,Housing,Bank,rent;bills,";

        std::fs::write(&path, csv_content).unwrap();

        let transactions = service.import_csv(&path).unwrap();

        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].description, "Salary");
        assert_eq!(transactions[0].amount, dec!(4500));
        assert!(matches!(
            transactions[0].transaction_type,
            TransactionType::Income
        ));
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(escape_csv("with\nnewline"), "\"with\nnewline\"");
    }

    #[test]
    fn test_exported_conversions() {
        // Test income conversion
        let income = IncomeSource::new(
            "salary".to_string(),
            "Job".to_string(),
            dec!(5000),
            Frequency::Monthly,
        );
        let exported: ExportedIncome = income.clone().into();
        assert_eq!(exported.variable_name, "salary");
        assert_eq!(exported.frequency, "Monthly");

        // Test expense conversion
        let expense = Expense::new(
            "rent".to_string(),
            "Rent".to_string(),
            dec!(1500),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        );
        let exported: ExportedExpense = expense.clone().into();
        assert_eq!(exported.variable_name, "rent");
        assert_eq!(exported.expense_type, "Fixed");

        // Test transaction conversion
        let tx = Transaction::new(
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            "Test".to_string(),
            dec!(100),
            TransactionType::Expense,
            "Test".to_string(),
        );
        let exported: ExportedTransaction = tx.clone().into();
        assert_eq!(exported.date, "2026-03-01");
        assert_eq!(exported.transaction_type, "Expense");

        // Test budget conversion
        let budget = Budget::new(3, 2026, "Food".to_string(), dec!(600));
        let exported: ExportedBudget = budget.clone().into();
        assert_eq!(exported.month, 3);
        assert_eq!(exported.year, 2026);
    }
}
