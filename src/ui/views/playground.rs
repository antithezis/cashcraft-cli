//! Playground view
//!
//! Expression calculator with:
//! - Expression input line
//! - Result display
//! - Variable sidebar (globals and locals)
//! - History panel

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::playground::session::{CalculationResult, PlaygroundSession};
use crate::domain::playground::Evaluator;
use crate::domain::playground::PlaygroundParser;
use crate::repository::Database;
use crate::services::{ExpenseService, IncomeService};
use crate::ui::theme::Theme;
use crate::ui::widgets::InputState;

/// State for the playground view
#[derive(Debug, Clone)]
pub struct PlaygroundState {
    /// Input state for expression entry
    pub input: InputState,
    /// Current session
    pub session: PlaygroundSession,
    /// Expression evaluator
    pub evaluator: Evaluator,
    /// Global variables from income/expenses
    pub globals: HashMap<String, Decimal>,
    /// Scroll offset for history
    pub history_scroll: usize,
    /// Is in insert mode
    pub insert_mode: bool,
}

impl Default for PlaygroundState {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaygroundState {
    /// Create new playground state
    pub fn new() -> Self {
        Self {
            input: InputState::new(),
            session: PlaygroundSession::new(),
            evaluator: Evaluator::new(),
            globals: HashMap::new(),
            history_scroll: 0,
            insert_mode: false,
        }
    }

    /// Refresh globals from database
    pub fn refresh(&mut self, db: &Database) {
        let income_service = IncomeService::new(db);
        let expense_service = ExpenseService::new(db);

        // Get playground variables from services
        let income_vars = income_service
            .get_playground_variables()
            .unwrap_or_default();
        let expense_vars = expense_service
            .get_playground_variables()
            .unwrap_or_default();

        // Merge into globals
        self.globals.clear();
        self.globals.extend(income_vars);
        self.globals.extend(expense_vars);

        // Update evaluator
        self.evaluator.set_globals(self.globals.clone());
    }

    /// Evaluate the current input expression
    pub fn evaluate(&mut self) {
        let input = self.input.value.trim().to_string();
        if input.is_empty() {
            return;
        }

        // Parse the input first
        match PlaygroundParser::parse_line(&input) {
            Ok(parsed_line) => {
                // Evaluate the parsed expression
                match self.evaluator.evaluate(parsed_line) {
                    Ok(result) => {
                        // Check if it was an assignment by looking at input
                        if input.contains('=') && !input.contains("==") {
                            // Try to extract variable name from input (simple approach)
                            if let Some(var_name) = input.split('=').next() {
                                let var_name = var_name.trim().to_string();
                                self.session
                                    .add_assignment_line(input.clone(), var_name, result);
                            } else {
                                self.session.add_value_line(input.clone(), result);
                            }
                        } else {
                            // Expression - show result
                            self.session.add_value_line(input.clone(), result);
                        }
                    }
                    Err(e) => {
                        self.session.add_error_line(input.clone(), e.to_string());
                    }
                }
            }
            Err(e) => {
                self.session.add_error_line(input.clone(), e.to_string());
            }
        }

        // Clear input
        self.input.clear();

        // Scroll to bottom
        self.history_scroll = self.session.lines.len().saturating_sub(1);
    }

    /// Clear the session
    pub fn clear(&mut self) {
        self.session.clear();
        self.evaluator.local_vars.clear();
        self.history_scroll = 0;
    }

    /// Get variable suggestions for autocomplete
    pub fn get_suggestions(&self, prefix: &str) -> Vec<String> {
        let prefix_lower = prefix.to_lowercase();
        let mut suggestions: Vec<String> = self
            .globals
            .keys()
            .filter(|k| k.to_lowercase().starts_with(&prefix_lower))
            .map(|k| format!("${}", k))
            .collect();

        // Add local variables
        for (k, _) in self.evaluator.local_vars.iter() {
            if k.to_lowercase().starts_with(&prefix_lower) {
                suggestions.push(k.clone());
            }
        }

        suggestions.sort();
        suggestions
    }
}

/// Playground view widget
pub struct PlaygroundView<'a> {
    state: &'a PlaygroundState,
    theme: &'a Theme,
}

impl<'a> PlaygroundView<'a> {
    /// Create new playground view
    pub fn new(state: &'a PlaygroundState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Render the variables sidebar
    fn render_variables(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Variables ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        // Split into globals and locals
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(inner);

        // Globals section
        let globals_block = Block::default()
            .title("Globals")
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(self.theme.colors.border));

        let globals_inner = globals_block.inner(chunks[0]);
        globals_block.render(chunks[0], buf);

        let mut sorted_globals: Vec<_> = self.state.globals.iter().collect();
        sorted_globals.sort_by_key(|(k, _)| k.as_str());

        for (i, (name, value)) in sorted_globals.iter().enumerate() {
            let y = globals_inner.y + i as u16;
            if y >= globals_inner.y + globals_inner.height {
                break;
            }

            let var_line = format!("${} = {:.2}", name, value);
            let truncated = if var_line.len() > globals_inner.width as usize {
                format!("{}...", &var_line[..globals_inner.width as usize - 3])
            } else {
                var_line
            };

            buf.set_string(
                globals_inner.x,
                y,
                &truncated,
                Style::default().fg(self.theme.colors.accent),
            );
        }

        // Locals section
        let locals_header =
            Paragraph::new("Locals").style(Style::default().fg(self.theme.colors.text_muted));
        locals_header.render(Rect::new(chunks[1].x, chunks[1].y, chunks[1].width, 1), buf);

        let local_vars: Vec<_> = self.state.evaluator.local_vars.iter().collect();
        for (i, (name, value)) in local_vars.iter().enumerate() {
            let y = chunks[1].y + 1 + i as u16;
            if y >= chunks[1].y + chunks[1].height {
                break;
            }

            let var_line = format!("{} = {:.2}", name, value);
            let truncated = if var_line.len() > chunks[1].width as usize {
                format!("{}...", &var_line[..chunks[1].width as usize - 3])
            } else {
                var_line
            };

            buf.set_string(
                chunks[1].x,
                y,
                &truncated,
                Style::default().fg(self.theme.colors.success),
            );
        }
    }

    /// Render the history/output panel
    fn render_history(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" History ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.state.session.lines.is_empty() {
            let hint = Paragraph::new("Type an expression and press Enter to evaluate")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            hint.render(inner, buf);
            return;
        }

        // Render history lines from bottom
        let visible_lines = inner.height as usize;
        let total_lines = self.state.session.lines.len();
        let start = total_lines.saturating_sub(visible_lines);

        for (i, line) in self.state.session.lines.iter().skip(start).enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            // PlaygroundLine has input, result (Option<CalculationResult>), line_number
            let (prefix, content, style) = match &line.result {
                Some(CalculationResult::Value(v)) => (
                    "= ",
                    format!("{}", v),
                    Style::default()
                        .fg(self.theme.colors.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Some(CalculationResult::Assignment { variable, value }) => (
                    "= ",
                    format!("{} = {}", variable, value),
                    Style::default().fg(self.theme.colors.accent),
                ),
                Some(CalculationResult::Error(e)) => (
                    "! ",
                    e.clone(),
                    Style::default().fg(self.theme.colors.error),
                ),
                None => (
                    "> ",
                    line.input.clone(),
                    Style::default().fg(self.theme.colors.text_primary),
                ),
            };

            let display = format!("{}{}", prefix, content);
            let truncated = if display.len() > inner.width as usize {
                format!("{}...", &display[..inner.width as usize - 3])
            } else {
                display
            };

            buf.set_string(inner.x, y, &truncated, style);
        }
    }

    /// Render the input line
    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(if self.state.insert_mode {
                " Input (INSERT) "
            } else {
                " Input "
            })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if self.state.insert_mode {
                self.theme.colors.accent
            } else {
                self.theme.colors.border
            }));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render prompt and input
        let prompt = "> ";
        buf.set_string(
            inner.x,
            inner.y,
            prompt,
            Style::default().fg(self.theme.colors.accent),
        );

        let input_area = Rect::new(
            inner.x + prompt.len() as u16,
            inner.y,
            inner.width.saturating_sub(prompt.len() as u16),
            1,
        );

        // Render input text
        let input_text = &self.state.input.value;
        buf.set_string(
            input_area.x,
            input_area.y,
            input_text,
            Style::default().fg(self.theme.colors.text_primary),
        );

        // Render cursor if in insert mode
        if self.state.insert_mode {
            let cursor_x = input_area.x + self.state.input.cursor as u16;
            if cursor_x < input_area.x + input_area.width {
                let cursor_char = input_text
                    .chars()
                    .nth(self.state.input.cursor)
                    .unwrap_or(' ');
                buf.set_string(
                    cursor_x,
                    input_area.y,
                    cursor_char.to_string(),
                    Style::default()
                        .bg(self.theme.colors.accent)
                        .fg(self.theme.colors.background),
                );
            }
        }
    }
}

impl Widget for PlaygroundView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout: sidebar | main (history + input)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(30)])
            .split(area);

        // Variables sidebar
        self.render_variables(main_chunks[0], buf);

        // Main area: history + input
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(main_chunks[1]);

        // History
        self.render_history(right_chunks[0], buf);

        // Input
        self.render_input(right_chunks[1], buf);
    }
}
