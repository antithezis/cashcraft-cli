//! Playground session management
//!
//! Manages playground sessions including:
//! - Session state with lines and results
//! - Local variable persistence
//! - History tracking
//! - Session save/load functionality

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A playground session containing calculation lines and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundSession {
    /// Unique session identifier
    pub id: Uuid,
    /// Optional name for saved sessions
    pub name: Option<String>,
    /// Lines of calculations with results
    pub lines: Vec<PlaygroundLine>,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
}

/// A single line in the playground session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundLine {
    /// The raw input text
    pub input: String,
    /// The result of evaluating the line
    pub result: Option<CalculationResult>,
    /// Line number (1-indexed)
    pub line_number: usize,
}

/// Result of evaluating a calculation line
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CalculationResult {
    /// Simple value result (e.g., 2 + 2 -> 4)
    Value(Decimal),
    /// Assignment result (e.g., x = 100 -> 100)
    Assignment {
        /// Variable name that was assigned
        variable: String,
        /// Value assigned to the variable
        value: Decimal,
    },
    /// Error during evaluation
    Error(String),
}

impl PlaygroundSession {
    /// Create a new empty session
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: None,
            lines: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new session with a name
    pub fn with_name(name: impl Into<String>) -> Self {
        let mut session = Self::new();
        session.name = Some(name.into());
        session
    }

    /// Add a new line to the session
    pub fn add_line(&mut self, input: String, result: Option<CalculationResult>) {
        let line_number = self.lines.len() + 1;
        self.lines.push(PlaygroundLine {
            input,
            result,
            line_number,
        });
        self.updated_at = Utc::now();
    }

    /// Add a line with a value result
    pub fn add_value_line(&mut self, input: impl Into<String>, value: Decimal) {
        self.add_line(input.into(), Some(CalculationResult::Value(value)));
    }

    /// Add a line with an assignment result
    pub fn add_assignment_line(
        &mut self,
        input: impl Into<String>,
        variable: impl Into<String>,
        value: Decimal,
    ) {
        self.add_line(
            input.into(),
            Some(CalculationResult::Assignment {
                variable: variable.into(),
                value,
            }),
        );
    }

    /// Add a line with an error result
    pub fn add_error_line(&mut self, input: impl Into<String>, error: impl Into<String>) {
        self.add_line(input.into(), Some(CalculationResult::Error(error.into())));
    }

    /// Get the last N lines
    pub fn last_lines(&self, count: usize) -> &[PlaygroundLine] {
        let start = self.lines.len().saturating_sub(count);
        &self.lines[start..]
    }

    /// Get a specific line by number (1-indexed)
    pub fn get_line(&self, line_number: usize) -> Option<&PlaygroundLine> {
        if line_number == 0 || line_number > self.lines.len() {
            None
        } else {
            self.lines.get(line_number - 1)
        }
    }

    /// Update a line at a specific position
    pub fn update_line(
        &mut self,
        line_number: usize,
        input: String,
        result: Option<CalculationResult>,
    ) -> bool {
        if line_number == 0 || line_number > self.lines.len() {
            return false;
        }
        let line = &mut self.lines[line_number - 1];
        line.input = input;
        line.result = result;
        self.updated_at = Utc::now();
        true
    }

    /// Remove a line at a specific position
    pub fn remove_line(&mut self, line_number: usize) -> bool {
        if line_number == 0 || line_number > self.lines.len() {
            return false;
        }
        self.lines.remove(line_number - 1);
        // Renumber remaining lines
        for (i, line) in self.lines.iter_mut().enumerate() {
            line.line_number = i + 1;
        }
        self.updated_at = Utc::now();
        true
    }

    /// Clear all lines from the session
    pub fn clear(&mut self) {
        self.lines.clear();
        self.updated_at = Utc::now();
    }

    /// Get the number of lines in the session
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Check if the session is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Get all successful value results
    pub fn values(&self) -> Vec<Decimal> {
        self.lines
            .iter()
            .filter_map(|line| match &line.result {
                Some(CalculationResult::Value(v)) => Some(*v),
                Some(CalculationResult::Assignment { value, .. }) => Some(*value),
                _ => None,
            })
            .collect()
    }

    /// Get all variable assignments
    pub fn assignments(&self) -> Vec<(&str, Decimal)> {
        self.lines
            .iter()
            .filter_map(|line| match &line.result {
                Some(CalculationResult::Assignment { variable, value }) => {
                    Some((variable.as_str(), *value))
                }
                _ => None,
            })
            .collect()
    }

    /// Get input history (all inputs)
    pub fn history(&self) -> Vec<&str> {
        self.lines.iter().map(|l| l.input.as_str()).collect()
    }

    /// Set session name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
        self.updated_at = Utc::now();
    }

    /// Clear session name
    pub fn clear_name(&mut self) {
        self.name = None;
        self.updated_at = Utc::now();
    }

    /// Convert session to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Create session from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Export session as plain text
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        if let Some(ref name) = self.name {
            output.push_str(&format!("# {}\n", name));
        }
        output.push_str(&format!(
            "# Created: {}\n\n",
            self.created_at.format("%Y-%m-%d %H:%M")
        ));

        for line in &self.lines {
            output.push_str(&line.input);
            if let Some(ref result) = line.result {
                match result {
                    CalculationResult::Value(v) => {
                        output.push_str(&format!("  # = {}", v));
                    }
                    CalculationResult::Assignment { variable, value } => {
                        output.push_str(&format!("  # {} = {}", variable, value));
                    }
                    CalculationResult::Error(e) => {
                        output.push_str(&format!("  # Error: {}", e));
                    }
                }
            }
            output.push('\n');
        }

        output
    }

    /// Export session as markdown
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        if let Some(ref name) = self.name {
            output.push_str(&format!("# {}\n\n", name));
        } else {
            output.push_str("# Playground Session\n\n");
        }
        output.push_str(&format!(
            "*Created: {}*\n\n",
            self.created_at.format("%Y-%m-%d %H:%M")
        ));
        output.push_str("```\n");

        for line in &self.lines {
            let result_str = match &line.result {
                Some(CalculationResult::Value(v)) => format!("{}", v),
                Some(CalculationResult::Assignment { value, .. }) => format!("{}", value),
                Some(CalculationResult::Error(e)) => format!("Error: {}", e),
                None => "...".to_string(),
            };
            output.push_str(&format!("{:<40} → {}\n", line.input, result_str));
        }

        output.push_str("```\n");
        output
    }
}

impl Default for PlaygroundSession {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaygroundLine {
    /// Create a new line with just input (no result yet)
    pub fn new(input: impl Into<String>, line_number: usize) -> Self {
        Self {
            input: input.into(),
            result: None,
            line_number,
        }
    }

    /// Check if the line has an error
    pub fn is_error(&self) -> bool {
        matches!(self.result, Some(CalculationResult::Error(_)))
    }

    /// Check if the line is an assignment
    pub fn is_assignment(&self) -> bool {
        matches!(self.result, Some(CalculationResult::Assignment { .. }))
    }

    /// Get the value if available
    pub fn value(&self) -> Option<Decimal> {
        match &self.result {
            Some(CalculationResult::Value(v)) => Some(*v),
            Some(CalculationResult::Assignment { value, .. }) => Some(*value),
            _ => None,
        }
    }
}

impl CalculationResult {
    /// Get the value if this is a successful result
    pub fn value(&self) -> Option<Decimal> {
        match self {
            CalculationResult::Value(v) => Some(*v),
            CalculationResult::Assignment { value, .. } => Some(*value),
            CalculationResult::Error(_) => None,
        }
    }

    /// Check if this is an error
    pub fn is_error(&self) -> bool {
        matches!(self, CalculationResult::Error(_))
    }

    /// Get error message if this is an error
    pub fn error(&self) -> Option<&str> {
        match self {
            CalculationResult::Error(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let session = PlaygroundSession::new();
        assert!(session.is_empty());
        assert!(session.name.is_none());
    }

    #[test]
    fn test_session_with_name() {
        let session = PlaygroundSession::with_name("My Session");
        assert_eq!(session.name, Some("My Session".to_string()));
    }

    #[test]
    fn test_add_line() {
        let mut session = PlaygroundSession::new();
        session.add_line(
            "2 + 2".to_string(),
            Some(CalculationResult::Value(Decimal::from(4))),
        );

        assert_eq!(session.line_count(), 1);
        assert_eq!(session.get_line(1).unwrap().input, "2 + 2");
    }

    #[test]
    fn test_add_value_line() {
        let mut session = PlaygroundSession::new();
        session.add_value_line("3 * 3", Decimal::from(9));

        let line = session.get_line(1).unwrap();
        assert_eq!(line.value(), Some(Decimal::from(9)));
    }

    #[test]
    fn test_add_assignment_line() {
        let mut session = PlaygroundSession::new();
        session.add_assignment_line("x = 100", "x", Decimal::from(100));

        let line = session.get_line(1).unwrap();
        assert!(line.is_assignment());
        assert_eq!(line.value(), Some(Decimal::from(100)));
    }

    #[test]
    fn test_add_error_line() {
        let mut session = PlaygroundSession::new();
        session.add_error_line("$unknown", "Variable not found");

        let line = session.get_line(1).unwrap();
        assert!(line.is_error());
    }

    #[test]
    fn test_remove_line() {
        let mut session = PlaygroundSession::new();
        session.add_value_line("1", Decimal::from(1));
        session.add_value_line("2", Decimal::from(2));
        session.add_value_line("3", Decimal::from(3));

        assert!(session.remove_line(2));
        assert_eq!(session.line_count(), 2);

        // Check line numbers were updated
        assert_eq!(session.get_line(1).unwrap().input, "1");
        assert_eq!(session.get_line(2).unwrap().input, "3");
    }

    #[test]
    fn test_clear() {
        let mut session = PlaygroundSession::new();
        session.add_value_line("1", Decimal::from(1));
        session.add_value_line("2", Decimal::from(2));

        session.clear();
        assert!(session.is_empty());
    }

    #[test]
    fn test_values() {
        let mut session = PlaygroundSession::new();
        session.add_value_line("1", Decimal::from(1));
        session.add_assignment_line("x = 2", "x", Decimal::from(2));
        session.add_error_line("err", "error");

        let values = session.values();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], Decimal::from(1));
        assert_eq!(values[1], Decimal::from(2));
    }

    #[test]
    fn test_assignments() {
        let mut session = PlaygroundSession::new();
        session.add_assignment_line("x = 1", "x", Decimal::from(1));
        session.add_assignment_line("y = 2", "y", Decimal::from(2));
        session.add_value_line("3", Decimal::from(3));

        let assignments = session.assignments();
        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments[0], ("x", Decimal::from(1)));
        assert_eq!(assignments[1], ("y", Decimal::from(2)));
    }

    #[test]
    fn test_history() {
        let mut session = PlaygroundSession::new();
        session.add_value_line("2 + 2", Decimal::from(4));
        session.add_value_line("3 * 3", Decimal::from(9));

        let history = session.history();
        assert_eq!(history, vec!["2 + 2", "3 * 3"]);
    }

    #[test]
    fn test_to_json() {
        let mut session = PlaygroundSession::with_name("Test");
        session.add_value_line("1 + 1", Decimal::from(2));

        let json = session.to_json().unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("1 + 1"));
    }

    #[test]
    fn test_from_json() {
        let mut session = PlaygroundSession::with_name("Test");
        session.add_value_line("1 + 1", Decimal::from(2));

        let json = session.to_json().unwrap();
        let restored = PlaygroundSession::from_json(&json).unwrap();

        assert_eq!(restored.name, Some("Test".to_string()));
        assert_eq!(restored.line_count(), 1);
    }

    #[test]
    fn test_to_text() {
        let mut session = PlaygroundSession::with_name("Budget Calc");
        session.add_assignment_line("x = 100", "x", Decimal::from(100));

        let text = session.to_text();
        assert!(text.contains("# Budget Calc"));
        assert!(text.contains("x = 100"));
    }

    #[test]
    fn test_to_markdown() {
        let mut session = PlaygroundSession::with_name("Budget Calc");
        session.add_value_line("2 + 2", Decimal::from(4));

        let md = session.to_markdown();
        assert!(md.contains("# Budget Calc"));
        assert!(md.contains("```"));
        assert!(md.contains("2 + 2"));
        assert!(md.contains("4"));
    }

    #[test]
    fn test_last_lines() {
        let mut session = PlaygroundSession::new();
        for i in 1..=5 {
            session.add_value_line(format!("{}", i), Decimal::from(i));
        }

        let last_3 = session.last_lines(3);
        assert_eq!(last_3.len(), 3);
        assert_eq!(last_3[0].input, "3");
        assert_eq!(last_3[1].input, "4");
        assert_eq!(last_3[2].input, "5");
    }
}
