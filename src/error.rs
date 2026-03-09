//! Error types for CashCraft

use thiserror::Error;

/// Main error type for CashCraft
#[derive(Error, Debug)]
pub enum CashCraftError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Variable '{0}' not found")]
    VariableNotFound(String),

    #[error("Variable name '{0}' is reserved")]
    ReservedVariableName(String),

    #[error("Variable name '{0}' already exists")]
    DuplicateVariableName(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
}

/// Convenience Result type for CashCraft operations
pub type Result<T> = std::result::Result<T, CashCraftError>;
