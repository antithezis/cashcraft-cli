//! Playground - Expression calculator with variable interpolation
//!
//! The Playground provides a calculation environment with:
//! - PEG expression parser (via Pest) for arithmetic expressions
//! - Variable system: $globals from income/expenses, locals from session
//! - Built-in functions: round, floor, ceil, abs, min, max, avg, sum, monthly, yearly
//! - Session management with history and persistence
//!
//! # Example
//!
//! ```
//! use cashcraft::domain::playground::{PlaygroundParser, Evaluator, ParsedLine};
//! use rust_decimal::Decimal;
//!
//! // Parse and evaluate an expression
//! let mut evaluator = Evaluator::new();
//!
//! // Set global variables (from income/expenses)
//! evaluator.set_global("salary", Decimal::from(4500));
//! evaluator.set_global("rent", Decimal::from(1500));
//!
//! // Parse a line
//! let parsed = PlaygroundParser::parse_line("$salary - $rent").unwrap();
//!
//! // Evaluate it
//! let result = evaluator.evaluate(parsed).unwrap();
//! assert_eq!(result, Decimal::from(3000));
//! ```

pub mod evaluator;
pub mod parser;
pub mod session;

// Re-export main types
pub use evaluator::Evaluator;
pub use parser::{BinaryOperator, Expr, ParsedLine, PlaygroundParser};
pub use session::{CalculationResult, PlaygroundLine, PlaygroundSession};
