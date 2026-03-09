//! Playground expression parser using Pest PEG grammar
//!
//! Parses playground expressions into an AST for evaluation.
//! Supports:
//! - Numeric literals (integers and decimals)
//! - Global variables ($salary, $rent)
//! - Local variables (x, tax_rate)
//! - Binary operators (+, -, *, /, %, ^)
//! - Function calls (round, min, max, etc.)
//! - Assignments (x = expr)

use pest::Parser;
use pest_derive::Parser;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::error::{CashCraftError, Result};

/// Pest parser for playground expressions
#[derive(Parser)]
#[grammar = "domain/playground/grammar.pest"]
pub struct PlaygroundParser;

/// Expression AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal (e.g., 42, 3.14, -100)
    Number(Decimal),
    /// Global variable reference (e.g., $salary, $income)
    GlobalVar(String),
    /// Local variable reference (e.g., x, tax_rate)
    LocalVar(String),
    /// Binary operation (e.g., a + b, x * y)
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    /// Function call (e.g., round(x, 2), min(a, b))
    FunctionCall { name: String, args: Vec<Expr> },
}

/// Binary operators supported in expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Sub,
    /// Multiplication (*)
    Mul,
    /// Division (/)
    Div,
    /// Modulo (%)
    Mod,
    /// Power/Exponentiation (^)
    Pow,
}

/// Result of parsing a playground line
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedLine {
    /// Variable assignment (e.g., x = 100)
    Assignment { name: String, expr: Expr },
    /// Standalone expression (e.g., $salary * 12)
    Expression(Expr),
}

impl PlaygroundParser {
    /// Parse a single line of playground input
    ///
    /// # Examples
    /// ```
    /// use cashcraft::domain::playground::PlaygroundParser;
    ///
    /// // Parse an expression
    /// let result = PlaygroundParser::parse_line("2 + 2").unwrap();
    ///
    /// // Parse an assignment
    /// let result = PlaygroundParser::parse_line("x = 100").unwrap();
    /// ```
    pub fn parse_line(input: &str) -> Result<ParsedLine> {
        let input = input.trim();
        if input.is_empty() {
            return Err(CashCraftError::InvalidExpression(
                "Empty expression".to_string(),
            ));
        }

        let pairs = Self::parse(Rule::line, input)
            .map_err(|e| CashCraftError::Parse(format!("Syntax error: {}", e)))?;

        let line_pair = pairs
            .into_iter()
            .next()
            .ok_or_else(|| CashCraftError::Parse("No input parsed".to_string()))?;

        Self::parse_line_inner(line_pair)
    }

    fn parse_line_inner(pair: pest::iterators::Pair<Rule>) -> Result<ParsedLine> {
        let inner = pair
            .into_inner()
            .next()
            .ok_or_else(|| CashCraftError::Parse("Empty line".to_string()))?;

        match inner.as_rule() {
            Rule::assignment => {
                let mut parts = inner.into_inner();
                let name = parts
                    .next()
                    .ok_or_else(|| CashCraftError::Parse("Missing variable name".to_string()))?
                    .as_str()
                    .to_string();
                let expr_pair = parts
                    .next()
                    .ok_or_else(|| CashCraftError::Parse("Missing expression".to_string()))?;
                let expr = Self::parse_expr(expr_pair)?;
                Ok(ParsedLine::Assignment { name, expr })
            }
            Rule::expr => {
                let expr = Self::parse_expr(inner)?;
                Ok(ParsedLine::Expression(expr))
            }
            _ => Err(CashCraftError::Parse(format!(
                "Unexpected rule: {:?}",
                inner.as_rule()
            ))),
        }
    }

    fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
        let mut inner = pair.into_inner().peekable();

        // Parse first term
        let first_term = inner
            .next()
            .ok_or_else(|| CashCraftError::Parse("Empty expression".to_string()))?;
        let mut left = Self::parse_term(first_term)?;

        // Parse remaining (add_op term)* pairs
        while let Some(op_pair) = inner.next() {
            if op_pair.as_rule() == Rule::add_op {
                let op = match op_pair.as_str() {
                    "+" => BinaryOperator::Add,
                    "-" => BinaryOperator::Sub,
                    _ => {
                        return Err(CashCraftError::Parse(format!(
                            "Unknown operator: {}",
                            op_pair.as_str()
                        )))
                    }
                };
                let right_pair = inner
                    .next()
                    .ok_or_else(|| CashCraftError::Parse("Missing operand".to_string()))?;
                let right = Self::parse_term(right_pair)?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            }
        }

        Ok(left)
    }

    fn parse_term(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
        let mut inner = pair.into_inner().peekable();

        // Parse first power
        let first_power = inner
            .next()
            .ok_or_else(|| CashCraftError::Parse("Empty term".to_string()))?;
        let mut left = Self::parse_power(first_power)?;

        // Parse remaining (mul_op power)* pairs
        while let Some(op_pair) = inner.next() {
            if op_pair.as_rule() == Rule::mul_op {
                let op = match op_pair.as_str() {
                    "*" => BinaryOperator::Mul,
                    "/" => BinaryOperator::Div,
                    "%" => BinaryOperator::Mod,
                    _ => {
                        return Err(CashCraftError::Parse(format!(
                            "Unknown operator: {}",
                            op_pair.as_str()
                        )))
                    }
                };
                let right_pair = inner
                    .next()
                    .ok_or_else(|| CashCraftError::Parse("Missing operand".to_string()))?;
                let right = Self::parse_power(right_pair)?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            }
        }

        Ok(left)
    }

    fn parse_power(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
        let mut inner = pair.into_inner().peekable();

        // Parse first atom
        let first_atom = inner
            .next()
            .ok_or_else(|| CashCraftError::Parse("Empty power expression".to_string()))?;
        let mut left = Self::parse_atom(first_atom)?;

        // Parse remaining (pow_op atom)* pairs
        while let Some(op_pair) = inner.next() {
            if op_pair.as_rule() == Rule::pow_op {
                let right_pair = inner
                    .next()
                    .ok_or_else(|| CashCraftError::Parse("Missing operand".to_string()))?;
                let right = Self::parse_atom(right_pair)?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::Pow,
                    right: Box::new(right),
                };
            }
        }

        Ok(left)
    }

    fn parse_atom(pair: pest::iterators::Pair<Rule>) -> Result<Expr> {
        let inner = pair
            .into_inner()
            .next()
            .ok_or_else(|| CashCraftError::Parse("Empty atom".to_string()))?;

        match inner.as_rule() {
            Rule::number => {
                let num_str = inner.as_str();
                let decimal = Decimal::from_str(num_str).map_err(|e| {
                    CashCraftError::Parse(format!("Invalid number '{}': {}", num_str, e))
                })?;
                Ok(Expr::Number(decimal))
            }
            Rule::global_var => {
                // Strip the leading '$'
                let name = inner.as_str()[1..].to_string();
                Ok(Expr::GlobalVar(name))
            }
            Rule::local_var => {
                let name = inner.as_str().to_string();
                Ok(Expr::LocalVar(name))
            }
            Rule::function_call => {
                let mut parts = inner.into_inner();
                let name = parts
                    .next()
                    .ok_or_else(|| CashCraftError::Parse("Missing function name".to_string()))?
                    .as_str()
                    .to_string();

                let args: Vec<Expr> = parts
                    .map(|arg| Self::parse_expr(arg))
                    .collect::<Result<Vec<_>>>()?;

                Ok(Expr::FunctionCall { name, args })
            }
            Rule::expr => {
                // Parenthesized expression
                Self::parse_expr(inner)
            }
            _ => Err(CashCraftError::Parse(format!(
                "Unexpected atom rule: {:?}",
                inner.as_rule()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let result = PlaygroundParser::parse_line("42").unwrap();
        match result {
            ParsedLine::Expression(Expr::Number(n)) => {
                assert_eq!(n, Decimal::from(42));
            }
            _ => panic!("Expected number expression"),
        }
    }

    #[test]
    fn test_parse_decimal() {
        let result = PlaygroundParser::parse_line("3.14").unwrap();
        match result {
            ParsedLine::Expression(Expr::Number(n)) => {
                assert_eq!(n, Decimal::from_str("3.14").unwrap());
            }
            _ => panic!("Expected decimal expression"),
        }
    }

    #[test]
    fn test_parse_negative_number() {
        let result = PlaygroundParser::parse_line("-100").unwrap();
        match result {
            ParsedLine::Expression(Expr::Number(n)) => {
                assert_eq!(n, Decimal::from(-100));
            }
            _ => panic!("Expected negative number expression"),
        }
    }

    #[test]
    fn test_parse_global_var() {
        let result = PlaygroundParser::parse_line("$salary").unwrap();
        match result {
            ParsedLine::Expression(Expr::GlobalVar(name)) => {
                assert_eq!(name, "salary");
            }
            _ => panic!("Expected global var expression"),
        }
    }

    #[test]
    fn test_parse_local_var() {
        let result = PlaygroundParser::parse_line("x").unwrap();
        match result {
            ParsedLine::Expression(Expr::LocalVar(name)) => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected local var expression"),
        }
    }

    #[test]
    fn test_parse_simple_addition() {
        let result = PlaygroundParser::parse_line("2 + 3").unwrap();
        match result {
            ParsedLine::Expression(Expr::BinaryOp { op, .. }) => {
                assert_eq!(op, BinaryOperator::Add);
            }
            _ => panic!("Expected binary op expression"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let result = PlaygroundParser::parse_line("x = 100").unwrap();
        match result {
            ParsedLine::Assignment { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let result = PlaygroundParser::parse_line("$salary - $rent + 1000").unwrap();
        assert!(matches!(result, ParsedLine::Expression(_)));
    }

    #[test]
    fn test_parse_function_call() {
        let result = PlaygroundParser::parse_line("round(3.14159, 2)").unwrap();
        match result {
            ParsedLine::Expression(Expr::FunctionCall { name, args }) => {
                assert_eq!(name, "round");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let result = PlaygroundParser::parse_line("(2 + 3) * 4").unwrap();
        assert!(matches!(result, ParsedLine::Expression(_)));
    }

    #[test]
    fn test_empty_input() {
        let result = PlaygroundParser::parse_line("");
        assert!(result.is_err());
    }

    #[test]
    fn test_operator_precedence() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let result = PlaygroundParser::parse_line("2 + 3 * 4").unwrap();
        match result {
            ParsedLine::Expression(Expr::BinaryOp { op, left, right }) => {
                // Top-level should be addition
                assert_eq!(op, BinaryOperator::Add);
                // Left should be 2
                assert!(matches!(*left, Expr::Number(_)));
                // Right should be 3 * 4
                assert!(matches!(
                    *right,
                    Expr::BinaryOp {
                        op: BinaryOperator::Mul,
                        ..
                    }
                ));
            }
            _ => panic!("Expected binary op expression"),
        }
    }
}
