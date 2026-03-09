//! Playground expression evaluator
//!
//! Evaluates parsed expressions with variable resolution and built-in functions.
//! Supports:
//! - Decimal arithmetic with rust_decimal
//! - Global variables from income/expenses ($salary, $rent, $income, $expenses)
//! - Local variables defined in the playground session
//! - 10 built-in functions: round, floor, ceil, abs, min, max, avg, sum, monthly, yearly

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;

use super::parser::{BinaryOperator, Expr, ParsedLine};
use crate::error::{CashCraftError, Result};

/// Expression evaluator with variable storage
///
/// The Evaluator maintains both global variables (from income/expenses)
/// and local variables (defined during playground session).
#[derive(Debug, Clone)]
pub struct Evaluator {
    /// Local variables defined in playground (e.g., x = 100)
    pub local_vars: HashMap<String, Decimal>,
    /// Global variables from income/expenses (e.g., $salary, $income)
    global_vars: HashMap<String, Decimal>,
}

impl Evaluator {
    /// Create a new evaluator with empty variable storage
    pub fn new() -> Self {
        Self {
            local_vars: HashMap::new(),
            global_vars: HashMap::new(),
        }
    }

    /// Set global variables (from income/expense sources)
    ///
    /// This should be called with all global variables before evaluation.
    /// Global variables are referenced with $ prefix in expressions.
    ///
    /// # Example
    /// ```
    /// use std::collections::HashMap;
    /// use rust_decimal::Decimal;
    /// use cashcraft::domain::playground::Evaluator;
    ///
    /// let mut evaluator = Evaluator::new();
    /// let mut globals = HashMap::new();
    /// globals.insert("salary".to_string(), Decimal::from(4500));
    /// globals.insert("income".to_string(), Decimal::from(5700));
    /// evaluator.set_globals(globals);
    /// ```
    pub fn set_globals(&mut self, globals: HashMap<String, Decimal>) {
        self.global_vars = globals;
    }

    /// Set a single global variable
    pub fn set_global(&mut self, name: impl Into<String>, value: Decimal) {
        self.global_vars.insert(name.into(), value);
    }

    /// Get a global variable value
    pub fn get_global(&self, name: &str) -> Option<Decimal> {
        self.global_vars.get(name).copied()
    }

    /// Set a local variable
    pub fn set_local(&mut self, name: impl Into<String>, value: Decimal) {
        self.local_vars.insert(name.into(), value);
    }

    /// Get a local variable value
    pub fn get_local(&self, name: &str) -> Option<Decimal> {
        self.local_vars.get(name).copied()
    }

    /// Get all global variable names (for autocomplete)
    pub fn global_var_names(&self) -> Vec<&String> {
        self.global_vars.keys().collect()
    }

    /// Get all local variable names (for autocomplete)
    pub fn local_var_names(&self) -> Vec<&String> {
        self.local_vars.keys().collect()
    }

    /// Evaluate a parsed line
    ///
    /// For assignments, stores the result in local variables.
    /// Returns the computed value.
    pub fn evaluate(&mut self, line: ParsedLine) -> Result<Decimal> {
        match line {
            ParsedLine::Assignment { name, expr } => {
                let value = self.eval_expr(&expr)?;
                self.local_vars.insert(name, value);
                Ok(value)
            }
            ParsedLine::Expression(expr) => self.eval_expr(&expr),
        }
    }

    /// Evaluate an expression recursively
    fn eval_expr(&self, expr: &Expr) -> Result<Decimal> {
        match expr {
            Expr::Number(n) => Ok(*n),

            Expr::GlobalVar(name) => self
                .global_vars
                .get(name)
                .copied()
                .ok_or_else(|| CashCraftError::VariableNotFound(format!("${}", name))),

            Expr::LocalVar(name) => self
                .local_vars
                .get(name)
                .copied()
                .ok_or_else(|| CashCraftError::VariableNotFound(name.clone())),

            Expr::BinaryOp { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.apply_op(l, op, r)
            }

            Expr::FunctionCall { name, args } => self.eval_function(name, args),
        }
    }

    /// Apply a binary operator to two values
    fn apply_op(&self, left: Decimal, op: &BinaryOperator, right: Decimal) -> Result<Decimal> {
        match op {
            BinaryOperator::Add => Ok(left + right),
            BinaryOperator::Sub => Ok(left - right),
            BinaryOperator::Mul => Ok(left * right),
            BinaryOperator::Div => {
                if right.is_zero() {
                    Err(CashCraftError::DivisionByZero)
                } else {
                    Ok(left / right)
                }
            }
            BinaryOperator::Mod => {
                if right.is_zero() {
                    Err(CashCraftError::DivisionByZero)
                } else {
                    Ok(left % right)
                }
            }
            BinaryOperator::Pow => {
                // Decimal doesn't have native pow, convert to f64
                let base: f64 = left.to_f64().unwrap_or(0.0);
                let exp: f64 = right.to_f64().unwrap_or(0.0);
                let result = base.powf(exp);

                if result.is_infinite() || result.is_nan() {
                    Err(CashCraftError::InvalidExpression(
                        "Power overflow or invalid result".to_string(),
                    ))
                } else {
                    Decimal::from_f64(result).ok_or_else(|| {
                        CashCraftError::InvalidExpression("Power result out of range".to_string())
                    })
                }
            }
        }
    }

    /// Evaluate a function call
    fn eval_function(&self, name: &str, args: &[Expr]) -> Result<Decimal> {
        let evaluated: Vec<Decimal> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<Vec<_>>>()?;

        match name {
            // round(x) -> round to integer
            // round(x, n) -> round to n decimal places
            "round" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "round requires at least 1 argument".to_string(),
                    ));
                }
                let n = evaluated[0];
                let places = evaluated
                    .get(1)
                    .map(|d| d.to_i32().unwrap_or(0) as u32)
                    .unwrap_or(0);
                Ok(n.round_dp(places))
            }

            // floor(x) -> round down to integer
            "floor" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "floor requires 1 argument".to_string(),
                    ));
                }
                Ok(evaluated[0].floor())
            }

            // ceil(x) -> round up to integer
            "ceil" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "ceil requires 1 argument".to_string(),
                    ));
                }
                Ok(evaluated[0].ceil())
            }

            // abs(x) -> absolute value
            "abs" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "abs requires 1 argument".to_string(),
                    ));
                }
                Ok(evaluated[0].abs())
            }

            // min(a, b, ...) -> minimum value
            "min" => evaluated.into_iter().min().ok_or_else(|| {
                CashCraftError::InvalidExpression("min requires at least 1 argument".to_string())
            }),

            // max(a, b, ...) -> maximum value
            "max" => evaluated.into_iter().max().ok_or_else(|| {
                CashCraftError::InvalidExpression("max requires at least 1 argument".to_string())
            }),

            // sum(a, b, ...) -> sum of all values
            "sum" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "sum requires at least 1 argument".to_string(),
                    ));
                }
                Ok(evaluated.into_iter().sum())
            }

            // avg(a, b, ...) -> average of all values
            "avg" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "avg requires at least 1 argument".to_string(),
                    ));
                }
                let count = evaluated.len();
                let sum: Decimal = evaluated.into_iter().sum();
                Ok(sum / Decimal::from(count))
            }

            // monthly(x) -> convert yearly to monthly (x/12)
            "monthly" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "monthly requires 1 argument".to_string(),
                    ));
                }
                Ok(evaluated[0] / Decimal::from(12))
            }

            // yearly(x) -> convert monthly to yearly (x*12)
            "yearly" => {
                if evaluated.is_empty() {
                    return Err(CashCraftError::InvalidExpression(
                        "yearly requires 1 argument".to_string(),
                    ));
                }
                Ok(evaluated[0] * Decimal::from(12))
            }

            _ => Err(CashCraftError::InvalidExpression(format!(
                "Unknown function: {}",
                name
            ))),
        }
    }

    /// Clear all local variables
    pub fn clear(&mut self) {
        self.local_vars.clear();
    }

    /// Clear everything (local and global variables)
    pub fn clear_all(&mut self) {
        self.local_vars.clear();
        self.global_vars.clear();
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::playground::PlaygroundParser;

    fn eval(evaluator: &mut Evaluator, input: &str) -> Result<Decimal> {
        let parsed = PlaygroundParser::parse_line(input)?;
        evaluator.evaluate(parsed)
    }

    #[test]
    fn test_simple_number() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "42").unwrap(), Decimal::from(42));
    }

    #[test]
    fn test_decimal_number() {
        let mut e = Evaluator::new();
        assert_eq!(
            eval(&mut e, "3.14").unwrap(),
            Decimal::from_str("3.14").unwrap()
        );
    }

    #[test]
    fn test_addition() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "2 + 3").unwrap(), Decimal::from(5));
    }

    #[test]
    fn test_subtraction() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "10 - 4").unwrap(), Decimal::from(6));
    }

    #[test]
    fn test_multiplication() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "3 * 4").unwrap(), Decimal::from(12));
    }

    #[test]
    fn test_division() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "10 / 2").unwrap(), Decimal::from(5));
    }

    #[test]
    fn test_division_by_zero() {
        let mut e = Evaluator::new();
        let result = eval(&mut e, "10 / 0");
        assert!(matches!(result, Err(CashCraftError::DivisionByZero)));
    }

    #[test]
    fn test_modulo() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "10 % 3").unwrap(), Decimal::from(1));
    }

    #[test]
    fn test_power() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "2 ^ 3").unwrap(), Decimal::from(8));
    }

    #[test]
    fn test_operator_precedence() {
        let mut e = Evaluator::new();
        // 2 + 3 * 4 = 2 + 12 = 14
        assert_eq!(eval(&mut e, "2 + 3 * 4").unwrap(), Decimal::from(14));
    }

    #[test]
    fn test_parentheses() {
        let mut e = Evaluator::new();
        // (2 + 3) * 4 = 5 * 4 = 20
        assert_eq!(eval(&mut e, "(2 + 3) * 4").unwrap(), Decimal::from(20));
    }

    #[test]
    fn test_global_var() {
        let mut e = Evaluator::new();
        e.set_global("salary", Decimal::from(4500));
        assert_eq!(eval(&mut e, "$salary").unwrap(), Decimal::from(4500));
    }

    #[test]
    fn test_global_var_not_found() {
        let mut e = Evaluator::new();
        let result = eval(&mut e, "$unknown");
        assert!(matches!(result, Err(CashCraftError::VariableNotFound(_))));
    }

    #[test]
    fn test_local_var_assignment() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "x = 100").unwrap(), Decimal::from(100));
        assert_eq!(e.get_local("x"), Some(Decimal::from(100)));
    }

    #[test]
    fn test_local_var_reference() {
        let mut e = Evaluator::new();
        eval(&mut e, "x = 100").unwrap();
        assert_eq!(eval(&mut e, "x * 2").unwrap(), Decimal::from(200));
    }

    #[test]
    fn test_complex_expression() {
        let mut e = Evaluator::new();
        e.set_global("salary", Decimal::from(4500));
        e.set_global("rent", Decimal::from(1500));
        // $salary - $rent = 4500 - 1500 = 3000
        assert_eq!(
            eval(&mut e, "$salary - $rent").unwrap(),
            Decimal::from(3000)
        );
    }

    #[test]
    fn test_round_integer() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "round(3.7)").unwrap(), Decimal::from(4));
    }

    #[test]
    fn test_round_decimal_places() {
        let mut e = Evaluator::new();
        assert_eq!(
            eval(&mut e, "round(3.14159, 2)").unwrap(),
            Decimal::from_str("3.14").unwrap()
        );
    }

    #[test]
    fn test_floor() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "floor(3.9)").unwrap(), Decimal::from(3));
    }

    #[test]
    fn test_ceil() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "ceil(3.1)").unwrap(), Decimal::from(4));
    }

    #[test]
    fn test_abs() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "abs(-42)").unwrap(), Decimal::from(42));
    }

    #[test]
    fn test_min() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "min(5, 3, 8)").unwrap(), Decimal::from(3));
    }

    #[test]
    fn test_max() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "max(5, 3, 8)").unwrap(), Decimal::from(8));
    }

    #[test]
    fn test_sum() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "sum(1, 2, 3, 4)").unwrap(), Decimal::from(10));
    }

    #[test]
    fn test_avg() {
        let mut e = Evaluator::new();
        assert_eq!(eval(&mut e, "avg(2, 4, 6)").unwrap(), Decimal::from(4));
    }

    #[test]
    fn test_monthly() {
        let mut e = Evaluator::new();
        // 1200 yearly / 12 = 100 monthly
        assert_eq!(eval(&mut e, "monthly(1200)").unwrap(), Decimal::from(100));
    }

    #[test]
    fn test_yearly() {
        let mut e = Evaluator::new();
        // 100 monthly * 12 = 1200 yearly
        assert_eq!(eval(&mut e, "yearly(100)").unwrap(), Decimal::from(1200));
    }

    #[test]
    fn test_unknown_function() {
        let mut e = Evaluator::new();
        let result = eval(&mut e, "unknown(5)");
        assert!(matches!(result, Err(CashCraftError::InvalidExpression(_))));
    }

    #[test]
    fn test_clear_local_vars() {
        let mut e = Evaluator::new();
        eval(&mut e, "x = 100").unwrap();
        assert!(e.get_local("x").is_some());
        e.clear();
        assert!(e.get_local("x").is_none());
    }

    #[test]
    fn test_full_workflow() {
        let mut e = Evaluator::new();

        // Set up globals
        e.set_global("salary", Decimal::from(4500));
        e.set_global("freelance", Decimal::from(1200));
        e.set_global("rent", Decimal::from(1500));

        // Define local variables
        eval(&mut e, "income = $salary + $freelance").unwrap();
        assert_eq!(e.get_local("income"), Some(Decimal::from(5700)));

        // Use locals and globals together
        let savings = eval(&mut e, "income - $rent").unwrap();
        assert_eq!(savings, Decimal::from(4200));

        // Project yearly savings
        let yearly_savings = eval(&mut e, "yearly(4200)").unwrap();
        assert_eq!(yearly_savings, Decimal::from(50400));
    }
}
