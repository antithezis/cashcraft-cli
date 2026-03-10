//! Data repository layer
//!
//! SQLite database access and persistence using the repository pattern.

pub mod balance_repo;
pub mod budget_repo;
pub mod database;
pub mod expense_repo;

pub use balance_repo::BalanceRepository;
pub use budget_repo::BudgetRepository;
pub mod income_repo;
pub mod transaction_repo;

// Note: BudgetRepository not yet implemented
pub use database::Database;
pub use expense_repo::ExpenseRepository;
pub use income_repo::IncomeRepository;
pub use transaction_repo::TransactionRepository;

use crate::error::Result;

/// Generic repository trait for CRUD operations.
///
/// This trait defines the standard operations for persisting domain entities.
/// Implementations should handle all database interactions transparently.
pub trait Repository<T> {
    /// Create a new entity in the database.
    ///
    /// # Arguments
    /// * `item` - The entity to create
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn create(&self, item: &T) -> Result<()>;

    /// Get an entity by its ID.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the entity
    ///
    /// # Returns
    /// * `Result<Option<T>>` - The entity if found, None otherwise
    fn get_by_id(&self, id: &str) -> Result<Option<T>>;

    /// Get all entities.
    ///
    /// # Returns
    /// * `Result<Vec<T>>` - All entities in the repository
    fn get_all(&self) -> Result<Vec<T>>;

    /// Update an existing entity.
    ///
    /// # Arguments
    /// * `item` - The entity with updated values
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn update(&self, item: &T) -> Result<()>;

    /// Delete an entity by its ID.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the entity to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    fn delete(&self, id: &str) -> Result<()>;
}
