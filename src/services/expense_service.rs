//! Expense service
//!
//! Business logic for managing expenses with validation, categorization,
//! and playground integration.

use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::expense::{Expense, ExpenseType};
use crate::error::{CashCraftError, Result};
use crate::repository::{Database, ExpenseRepository, IncomeRepository, Repository};
use crate::services::income_service::is_reserved_variable;

/// Service for managing expenses.
///
/// Provides CRUD operations with validation, ensures variable name uniqueness
/// across both income and expense tables, and integrates with the playground
/// variable system.
pub struct ExpenseService<'a> {
    repo: ExpenseRepository<'a>,
    income_repo: IncomeRepository<'a>,
}

impl<'a> ExpenseService<'a> {
    /// Create a new ExpenseService with a database reference.
    pub fn new(db: &'a Database) -> Self {
        Self {
            repo: ExpenseRepository::new(db),
            income_repo: IncomeRepository::new(db),
        }
    }

    /// Create a new expense with validation.
    ///
    /// Validates the variable name format, checks for reserved names,
    /// and ensures uniqueness across income and expense tables.
    ///
    /// # Arguments
    /// * `expense` - The expense to create
    ///
    /// # Returns
    /// * `Result<()>` - Success or validation error
    pub fn create(&self, expense: &Expense) -> Result<()> {
        // Validate variable name format
        self.validate_variable_name(&expense.variable_name)?;

        // Check for reserved names
        if is_reserved_variable(&expense.variable_name) {
            return Err(CashCraftError::ReservedVariableName(
                expense.variable_name.clone(),
            ));
        }

        // Check uniqueness in expenses table
        if self
            .repo
            .get_by_variable_name(&expense.variable_name)?
            .is_some()
        {
            return Err(CashCraftError::DuplicateVariableName(
                expense.variable_name.clone(),
            ));
        }

        // Check uniqueness in income_sources table (cross-table constraint)
        if self
            .income_repo
            .get_by_variable_name(&expense.variable_name)?
            .is_some()
        {
            return Err(CashCraftError::DuplicateVariableName(
                expense.variable_name.clone(),
            ));
        }

        self.repo.create(expense)
    }

    /// Get all expenses.
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - All expenses
    pub fn get_all(&self) -> Result<Vec<Expense>> {
        self.repo.get_all()
    }

    /// Get only active expenses.
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - All active expenses
    pub fn get_active(&self) -> Result<Vec<Expense>> {
        self.repo.get_active()
    }

    /// Get an expense by ID.
    ///
    /// # Arguments
    /// * `id` - The expense ID
    ///
    /// # Returns
    /// * `Result<Option<Expense>>` - The expense if found
    pub fn get_by_id(&self, id: &str) -> Result<Option<Expense>> {
        self.repo.get_by_id(id)
    }

    /// Get an expense by variable name.
    ///
    /// # Arguments
    /// * `variable_name` - The variable name (e.g., "rent")
    ///
    /// # Returns
    /// * `Result<Option<Expense>>` - The expense if found
    pub fn get_by_variable_name(&self, variable_name: &str) -> Result<Option<Expense>> {
        self.repo.get_by_variable_name(variable_name)
    }

    /// Calculate total monthly expenses from all active sources.
    ///
    /// Converts each expense to its monthly equivalent based on frequency.
    ///
    /// # Returns
    /// * `Result<Decimal>` - Total monthly expenses
    pub fn total_monthly_expenses(&self) -> Result<Decimal> {
        let active = self.get_active()?;
        Ok(active.iter().map(|e| e.monthly_amount()).sum())
    }

    /// Get global variables for the playground.
    ///
    /// Returns a HashMap where each expense is available as `$variable_name`
    /// with its monthly equivalent value. Also includes `$expenses` as the total
    /// of all active expenses.
    ///
    /// # Returns
    /// * `Result<HashMap<String, Decimal>>` - Variable name to monthly amount mapping
    pub fn get_playground_variables(&self) -> Result<HashMap<String, Decimal>> {
        let active = self.get_active()?;
        let mut vars = HashMap::new();

        // Add individual expenses
        for expense in &active {
            vars.insert(expense.variable_name.clone(), expense.monthly_amount());
        }

        // Add $expenses aggregate
        let total: Decimal = active.iter().map(|e| e.monthly_amount()).sum();
        vars.insert("expenses".to_string(), total);

        Ok(vars)
    }

    /// Update an existing expense.
    ///
    /// # Arguments
    /// * `expense` - The expense with updated values
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn update(&self, expense: &Expense) -> Result<()> {
        // If variable name changed, validate the new name
        if let Some(existing) = self.repo.get_by_id(&expense.id.to_string())? {
            if existing.variable_name != expense.variable_name {
                self.validate_variable_name(&expense.variable_name)?;

                if is_reserved_variable(&expense.variable_name) {
                    return Err(CashCraftError::ReservedVariableName(
                        expense.variable_name.clone(),
                    ));
                }

                // Check uniqueness excluding current record
                if self
                    .repo
                    .get_by_variable_name(&expense.variable_name)?
                    .is_some()
                {
                    return Err(CashCraftError::DuplicateVariableName(
                        expense.variable_name.clone(),
                    ));
                }

                if self
                    .income_repo
                    .get_by_variable_name(&expense.variable_name)?
                    .is_some()
                {
                    return Err(CashCraftError::DuplicateVariableName(
                        expense.variable_name.clone(),
                    ));
                }
            }
        }

        self.repo.update(expense)
    }

    /// Delete an expense.
    ///
    /// # Arguments
    /// * `id` - The expense ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn delete(&self, id: &str) -> Result<()> {
        self.repo.delete(id)
    }

    /// Toggle the active state of an expense.
    ///
    /// # Arguments
    /// * `id` - The expense ID
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub fn toggle_active(&self, id: &str) -> Result<()> {
        if let Some(mut expense) = self.repo.get_by_id(id)? {
            expense.is_active = !expense.is_active;
            self.repo.update(&expense)?;
        }
        Ok(())
    }

    /// Get expenses grouped by category.
    ///
    /// # Returns
    /// * `Result<HashMap<ExpenseCategory, Vec<Expense>>>` - Category to expenses mapping
    pub fn get_grouped_by_category(&self) -> Result<HashMap<String, Vec<Expense>>> {
        let all = self.get_all()?;
        let mut grouped: HashMap<String, Vec<Expense>> = HashMap::new();

        for expense in all {
            let category = expense.category.as_str().to_string();
            grouped.entry(category).or_default().push(expense);
        }

        Ok(grouped)
    }

    /// Get expenses grouped by type (Fixed, Variable, OneTime).
    ///
    /// # Returns
    /// * `Result<HashMap<ExpenseType, Vec<Expense>>>` - Type to expenses mapping
    pub fn get_grouped_by_type(&self) -> Result<HashMap<String, Vec<Expense>>> {
        let all = self.get_all()?;
        let mut grouped: HashMap<String, Vec<Expense>> = HashMap::new();

        for expense in all {
            let type_name = match expense.expense_type {
                ExpenseType::Fixed => "Fixed",
                ExpenseType::Variable => "Variable",
                ExpenseType::OneTime => "OneTime",
            };
            grouped
                .entry(type_name.to_string())
                .or_default()
                .push(expense);
        }

        Ok(grouped)
    }

    /// Get expenses by category.
    ///
    /// # Arguments
    /// * `category` - The category to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - Expenses in the specified category
    pub fn get_by_category(&self, category: &str) -> Result<Vec<Expense>> {
        self.repo.get_by_category(category)
    }

    /// Get expenses by type.
    ///
    /// # Arguments
    /// * `expense_type` - The type to filter by
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - Expenses of the specified type
    pub fn get_by_type(&self, expense_type: &ExpenseType) -> Result<Vec<Expense>> {
        self.repo.get_by_type(expense_type)
    }

    /// Get only essential expenses.
    ///
    /// Essential expenses are those marked as required for basic needs.
    ///
    /// # Returns
    /// * `Result<Vec<Expense>>` - All essential active expenses
    pub fn get_essential(&self) -> Result<Vec<Expense>> {
        self.repo.get_essential()
    }

    /// Calculate total monthly fixed expenses.
    ///
    /// # Returns
    /// * `Result<Decimal>` - Total monthly fixed expenses
    pub fn total_fixed_expenses(&self) -> Result<Decimal> {
        let fixed = self.repo.get_by_type(&ExpenseType::Fixed)?;
        Ok(fixed
            .iter()
            .filter(|e| e.is_active)
            .map(|e| e.monthly_amount())
            .sum())
    }

    /// Calculate total monthly variable expenses.
    ///
    /// # Returns
    /// * `Result<Decimal>` - Total monthly variable expenses
    pub fn total_variable_expenses(&self) -> Result<Decimal> {
        let variable = self.repo.get_by_type(&ExpenseType::Variable)?;
        Ok(variable
            .iter()
            .filter(|e| e.is_active)
            .map(|e| e.monthly_amount())
            .sum())
    }

    /// Calculate category totals for active expenses.
    ///
    /// # Returns
    /// * `Result<HashMap<String, Decimal>>` - Category to total amount mapping
    pub fn get_category_totals(&self) -> Result<HashMap<String, Decimal>> {
        let active = self.get_active()?;
        let mut totals: HashMap<String, Decimal> = HashMap::new();

        for expense in active {
            let category = expense.category.as_str().to_string();
            *totals.entry(category).or_insert(Decimal::ZERO) += expense.monthly_amount();
        }

        Ok(totals)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::expense::ExpenseCategory;
    use crate::domain::income::Frequency;
    use crate::repository::Database;
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
    fn test_create_expense() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);
        let expense = create_test_expense();

        let result = service.create(&expense);
        assert!(result.is_ok());

        let retrieved = service.get_by_id(&expense.id.to_string()).unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_reserved_name_rejected() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);
        let expense = Expense::new(
            "expenses".to_string(),
            "Reserved".to_string(),
            dec!(1000),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        );

        let result = service.create(&expense);
        assert!(matches!(
            result,
            Err(CashCraftError::ReservedVariableName(_))
        ));
    }

    #[test]
    fn test_duplicate_name_rejected() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);

        let expense1 = create_test_expense();
        service.create(&expense1).unwrap();

        let expense2 = Expense::new(
            "rent".to_string(),
            "Another Rent".to_string(),
            dec!(1200),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        );

        let result = service.create(&expense2);
        assert!(matches!(
            result,
            Err(CashCraftError::DuplicateVariableName(_))
        ));
    }

    #[test]
    fn test_cross_table_uniqueness() {
        let db = create_test_db();

        // Create an income source with name "salary"
        let income_repo = IncomeRepository::new(&db);
        use crate::domain::income::IncomeSource;
        let income = IncomeSource::new(
            "salary".to_string(),
            "Primary Job".to_string(),
            dec!(4500),
            Frequency::Monthly,
        );
        income_repo.create(&income).unwrap();

        // Try to create expense with same name
        let expense_service = ExpenseService::new(&db);
        let expense = Expense::new(
            "salary".to_string(),
            "Salary Expense".to_string(),
            dec!(100),
            ExpenseType::Variable,
            Frequency::Monthly,
            ExpenseCategory::Custom("Test".to_string()),
        );

        let result = expense_service.create(&expense);
        assert!(matches!(
            result,
            Err(CashCraftError::DuplicateVariableName(_))
        ));
    }

    #[test]
    fn test_total_monthly_expenses() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);

        let expense1 = Expense::new(
            "rent".to_string(),
            "Rent".to_string(),
            dec!(1500),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Housing,
        );
        let expense2 = Expense::new(
            "groceries".to_string(),
            "Food".to_string(),
            dec!(150),
            ExpenseType::Variable,
            Frequency::Weekly, // 600/month
            ExpenseCategory::Food,
        );

        service.create(&expense1).unwrap();
        service.create(&expense2).unwrap();

        let total = service.total_monthly_expenses().unwrap();
        assert_eq!(total, dec!(2100)); // 1500 + 600
    }

    #[test]
    fn test_playground_variables() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);

        let expense1 = create_test_expense();
        let expense2 = Expense::new(
            "utilities".to_string(),
            "Utilities".to_string(),
            dec!(200),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Utilities,
        );

        service.create(&expense1).unwrap();
        service.create(&expense2).unwrap();

        let vars = service.get_playground_variables().unwrap();

        assert_eq!(vars.get("rent"), Some(&dec!(1500)));
        assert_eq!(vars.get("utilities"), Some(&dec!(200)));
        assert_eq!(vars.get("expenses"), Some(&dec!(1700)));
    }

    #[test]
    fn test_grouped_by_category() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);

        let expense1 = create_test_expense();
        let expense2 = Expense::new(
            "groceries".to_string(),
            "Food".to_string(),
            dec!(600),
            ExpenseType::Variable,
            Frequency::Monthly,
            ExpenseCategory::Food,
        );

        service.create(&expense1).unwrap();
        service.create(&expense2).unwrap();

        let grouped = service.get_grouped_by_category().unwrap();
        assert!(grouped.contains_key("Housing"));
        assert!(grouped.contains_key("Food"));
    }

    #[test]
    fn test_grouped_by_type() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);

        let expense1 = create_test_expense(); // Fixed
        let expense2 = Expense::new(
            "groceries".to_string(),
            "Food".to_string(),
            dec!(600),
            ExpenseType::Variable,
            Frequency::Monthly,
            ExpenseCategory::Food,
        );

        service.create(&expense1).unwrap();
        service.create(&expense2).unwrap();

        let grouped = service.get_grouped_by_type().unwrap();
        assert_eq!(grouped.get("Fixed").map(|v| v.len()), Some(1));
        assert_eq!(grouped.get("Variable").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_toggle_active() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);
        let expense = create_test_expense();

        service.create(&expense).unwrap();
        assert!(
            service
                .get_by_id(&expense.id.to_string())
                .unwrap()
                .unwrap()
                .is_active
        );

        service.toggle_active(&expense.id.to_string()).unwrap();
        assert!(
            !service
                .get_by_id(&expense.id.to_string())
                .unwrap()
                .unwrap()
                .is_active
        );
    }

    #[test]
    fn test_category_totals() {
        let db = create_test_db();
        let service = ExpenseService::new(&db);

        let expense1 = create_test_expense(); // Housing 1500
        let expense2 = Expense::new(
            "utilities".to_string(),
            "Utilities".to_string(),
            dec!(200),
            ExpenseType::Fixed,
            Frequency::Monthly,
            ExpenseCategory::Utilities,
        );
        let expense3 = Expense::new(
            "groceries".to_string(),
            "Food".to_string(),
            dec!(600),
            ExpenseType::Variable,
            Frequency::Monthly,
            ExpenseCategory::Food,
        );

        service.create(&expense1).unwrap();
        service.create(&expense2).unwrap();
        service.create(&expense3).unwrap();

        let totals = service.get_category_totals().unwrap();
        assert_eq!(totals.get("Housing"), Some(&dec!(1500)));
        assert_eq!(totals.get("Utilities"), Some(&dec!(200)));
        assert_eq!(totals.get("Food"), Some(&dec!(600)));
    }
}
