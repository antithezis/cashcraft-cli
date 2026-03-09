//! Income service
//!
//! Business logic for managing income sources with validation and playground integration.

use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::income::IncomeSource;
use crate::error::{CashCraftError, Result};
use crate::repository::{Database, ExpenseRepository, IncomeRepository, Repository};

/// Reserved variable names that cannot be used for income/expense sources.
const RESERVED_NAMES: &[&str] = &["income", "expenses", "spents", "savings", "balance"];

/// Service for managing income sources.
///
/// Provides CRUD operations with validation, ensures variable name uniqueness
/// across both income and expense tables, and integrates with the playground
/// variable system.
pub struct IncomeService<'a> {
    repo: IncomeRepository<'a>,
    expense_repo: ExpenseRepository<'a>,
}

impl<'a> IncomeService<'a> {
    /// Create a new IncomeService with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self {
            repo: IncomeRepository::new(db),
            expense_repo: ExpenseRepository::new(db),
        }
    }

    /// Create a new income source with validation.
    ///
    /// Validates the variable name format, checks for reserved names,
    /// and ensures uniqueness across income and expense tables.
    ///
    /// # Arguments
    /// * `income` - The income source to create
    ///
    /// # Returns
    /// * `Result<()>` - Success or validation error
    pub fn create(&self, income: &IncomeSource) -> Result<()> {
        // Validate variable name format
        self.validate_variable_name(&income.variable_name)?;

        // Check for reserved names
        if is_reserved_variable(&income.variable_name) {
            return Err(CashCraftError::ReservedVariableName(
                income.variable_name.clone(),
            ));
        }

        // Check uniqueness in income_sources table
        if self
            .repo
            .get_by_variable_name(&income.variable_name)?
            .is_some()
        {
            return Err(CashCraftError::DuplicateVariableName(
                income.variable_name.clone(),
            ));
        }

        // Check uniqueness in expenses table (cross-table constraint)
        if self
            .expense_repo
            .get_by_variable_name(&income.variable_name)?
            .is_some()
        {
            return Err(CashCraftError::DuplicateVariableName(
                income.variable_name.clone(),
            ));
        }

        self.repo.create(income)
    }

    /// Get all income sources.
    ///
    /// # Returns
    /// * `Result<Vec<IncomeSource>>` - All income sources
    pub fn get_all(&self) -> Result<Vec<IncomeSource>> {
        self.repo.get_all()
    }

    /// Get only active income sources.
    ///
    /// # Returns
    /// * `Result<Vec<IncomeSource>>` - All active income sources
    pub fn get_active(&self) -> Result<Vec<IncomeSource>> {
        self.repo.get_active()
    }

    /// Get an income source by ID.
    ///
    /// # Arguments
    /// * `id` - The income source ID
    ///
    /// # Returns
    /// * `Result<Option<IncomeSource>>` - The income source if found
    pub fn get_by_id(&self, id: &str) -> Result<Option<IncomeSource>> {
        self.repo.get_by_id(id)
    }

    /// Get an income source by variable name.
    ///
    /// # Arguments
    /// * `variable_name` - The variable name (e.g., "salary")
    ///
    /// # Returns
    /// * `Result<Option<IncomeSource>>` - The income source if found
    pub fn get_by_variable_name(&self, variable_name: &str) -> Result<Option<IncomeSource>> {
        self.repo.get_by_variable_name(variable_name)
    }

    /// Calculate total monthly income from all active sources.
    ///
    /// Converts each income source to its monthly equivalent based on frequency.
    ///
    /// # Returns
    /// * `Result<Decimal>` - Total monthly income
    pub fn total_monthly_income(&self) -> Result<Decimal> {
        let active = self.get_active()?;
        Ok(active.iter().map(|i| i.monthly_amount()).sum())
    }

    /// Get global variables for the playground.
    ///
    /// Returns a HashMap where each income source is available as `$variable_name`
    /// with its monthly equivalent value. Also includes `$income` as the total
    /// of all active income sources.
    ///
    /// # Returns
    /// * `Result<HashMap<String, Decimal>>` - Variable name to monthly amount mapping
    pub fn get_playground_variables(&self) -> Result<HashMap<String, Decimal>> {
        let active = self.get_active()?;
        let mut vars = HashMap::new();

        // Add individual income sources
        for income in &active {
            vars.insert(income.variable_name.clone(), income.monthly_amount());
        }

        // Add $income aggregate
        let total: Decimal = active.iter().map(|i| i.monthly_amount()).sum();
        vars.insert("income".to_string(), total);

        Ok(vars)
    }

    /// Update an existing income source.
    ///
    /// # Arguments
    /// * `income` - The income source with updated values
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn update(&self, income: &IncomeSource) -> Result<()> {
        // If variable name changed, validate the new name
        if let Some(existing) = self.repo.get_by_id(&income.id.to_string())? {
            if existing.variable_name != income.variable_name {
                self.validate_variable_name(&income.variable_name)?;

                if is_reserved_variable(&income.variable_name) {
                    return Err(CashCraftError::ReservedVariableName(
                        income.variable_name.clone(),
                    ));
                }

                // Check uniqueness excluding current record
                if self
                    .repo
                    .get_by_variable_name(&income.variable_name)?
                    .is_some()
                {
                    return Err(CashCraftError::DuplicateVariableName(
                        income.variable_name.clone(),
                    ));
                }

                if self
                    .expense_repo
                    .get_by_variable_name(&income.variable_name)?
                    .is_some()
                {
                    return Err(CashCraftError::DuplicateVariableName(
                        income.variable_name.clone(),
                    ));
                }
            }
        }

        self.repo.update(income)
    }

    /// Delete an income source.
    ///
    /// # Arguments
    /// * `id` - The income source ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn delete(&self, id: &str) -> Result<()> {
        self.repo.delete(id)
    }

    /// Toggle the active state of an income source.
    ///
    /// # Arguments
    /// * `id` - The income source ID
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn toggle_active(&self, id: &str) -> Result<()> {
        if let Some(mut income) = self.repo.get_by_id(id)? {
            income.is_active = !income.is_active;
            self.repo.update(&income)?;
        }
        Ok(())
    }

    /// Get income sources grouped by category.
    ///
    /// # Returns
    /// * `Result<HashMap<String, Vec<IncomeSource>>>` - Category to income sources mapping
    pub fn get_by_category(&self) -> Result<HashMap<String, Vec<IncomeSource>>> {
        let all = self.get_all()?;
        let mut grouped: HashMap<String, Vec<IncomeSource>> = HashMap::new();

        for income in all {
            let category = income
                .category
                .clone()
                .unwrap_or_else(|| "Uncategorized".to_string());
            grouped.entry(category).or_default().push(income);
        }

        Ok(grouped)
    }

    /// Validate variable name format.
    ///
    /// Variable names must:
    /// - Be 1-32 characters long
    /// - Contain only alphanumeric characters and underscores
    /// - Not start with a digit
    fn validate_variable_name(&self, name: &str) -> Result<()> {
        if name.is_empty() || name.len() > 32 {
            return Err(CashCraftError::Validation(
                "Variable name must be 1-32 characters".into(),
            ));
        }

        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(CashCraftError::Validation(
                "Variable name can only contain letters, numbers, and underscores".into(),
            ));
        }

        if name
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(true)
        {
            return Err(CashCraftError::Validation(
                "Variable name cannot start with a digit".into(),
            ));
        }

        Ok(())
    }
}

/// Check if a variable name is reserved.
///
/// Reserved names are used for system aggregates in the playground.
pub fn is_reserved_variable(name: &str) -> bool {
    RESERVED_NAMES.contains(&name.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::income::Frequency;
    use crate::repository::Database;
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
    fn test_create_income() {
        let db = create_test_db();
        let service = IncomeService::new(&db);
        let income = create_test_income();

        let result = service.create(&income);
        assert!(result.is_ok());

        let retrieved = service.get_by_id(&income.id.to_string()).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_reserved_name_rejected() {
        let db = create_test_db();
        let service = IncomeService::new(&db);
        let income = IncomeSource::new(
            "income".to_string(),
            "Reserved".to_string(),
            dec!(1000),
            Frequency::Monthly,
        );

        let result = service.create(&income);
        assert!(matches!(
            result,
            Err(CashCraftError::ReservedVariableName(_))
        ));
    }

    #[test]
    fn test_duplicate_name_rejected() {
        let db = create_test_db();
        let service = IncomeService::new(&db);

        let income1 = create_test_income();
        service.create(&income1).unwrap();

        let income2 = IncomeSource::new(
            "salary".to_string(),
            "Another Salary".to_string(),
            dec!(3000),
            Frequency::Monthly,
        );

        let result = service.create(&income2);
        assert!(matches!(
            result,
            Err(CashCraftError::DuplicateVariableName(_))
        ));
    }

    #[test]
    fn test_invalid_variable_name_rejected() {
        let db = create_test_db();
        let service = IncomeService::new(&db);

        // Name starting with digit
        let income = IncomeSource::new(
            "1salary".to_string(),
            "Invalid".to_string(),
            dec!(1000),
            Frequency::Monthly,
        );
        assert!(service.create(&income).is_err());

        // Name with invalid characters
        let income = IncomeSource::new(
            "my-salary".to_string(),
            "Invalid".to_string(),
            dec!(1000),
            Frequency::Monthly,
        );
        assert!(service.create(&income).is_err());

        // Empty name
        let income = IncomeSource::new(
            "".to_string(),
            "Invalid".to_string(),
            dec!(1000),
            Frequency::Monthly,
        );
        assert!(service.create(&income).is_err());
    }

    #[test]
    fn test_total_monthly_income() {
        let db = create_test_db();
        let service = IncomeService::new(&db);

        let income1 = IncomeSource::new(
            "salary".to_string(),
            "Primary".to_string(),
            dec!(4500),
            Frequency::Monthly,
        );
        let income2 = IncomeSource::new(
            "freelance".to_string(),
            "Side Work".to_string(),
            dec!(1000),
            Frequency::BiWeekly, // 2000/month
        );

        service.create(&income1).unwrap();
        service.create(&income2).unwrap();

        let total = service.total_monthly_income().unwrap();
        assert_eq!(total, dec!(6500)); // 4500 + 2000
    }

    #[test]
    fn test_playground_variables() {
        let db = create_test_db();
        let service = IncomeService::new(&db);

        let income1 = IncomeSource::new(
            "salary".to_string(),
            "Primary".to_string(),
            dec!(4500),
            Frequency::Monthly,
        );
        let income2 = IncomeSource::new(
            "freelance".to_string(),
            "Side Work".to_string(),
            dec!(1200),
            Frequency::Monthly,
        );

        service.create(&income1).unwrap();
        service.create(&income2).unwrap();

        let vars = service.get_playground_variables().unwrap();

        assert_eq!(vars.get("salary"), Some(&dec!(4500)));
        assert_eq!(vars.get("freelance"), Some(&dec!(1200)));
        assert_eq!(vars.get("income"), Some(&dec!(5700)));
    }

    #[test]
    fn test_toggle_active() {
        let db = create_test_db();
        let service = IncomeService::new(&db);
        let income = create_test_income();

        service.create(&income).unwrap();
        assert!(
            service
                .get_by_id(&income.id.to_string())
                .unwrap()
                .unwrap()
                .is_active
        );

        service.toggle_active(&income.id.to_string()).unwrap();
        assert!(
            !service
                .get_by_id(&income.id.to_string())
                .unwrap()
                .unwrap()
                .is_active
        );
    }

    #[test]
    fn test_is_reserved_variable() {
        assert!(is_reserved_variable("income"));
        assert!(is_reserved_variable("INCOME"));
        assert!(is_reserved_variable("Income"));
        assert!(is_reserved_variable("expenses"));
        assert!(is_reserved_variable("savings"));
        assert!(is_reserved_variable("balance"));
        assert!(!is_reserved_variable("salary"));
        assert!(!is_reserved_variable("rent"));
    }
}
