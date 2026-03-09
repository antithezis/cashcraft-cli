//! Expenses view
//!
//! Displays and manages expense sources with:
//! - Expense list
//! - Add/Edit/Delete operations
//! - Variable name display ($name)
//! - Category and type display

use crate::ui::widgets::{InputState, TextInput};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use rust_decimal::Decimal;

use crate::domain::expense::{Expense, ExpenseCategory, ExpenseType};
use crate::repository::Database;
use crate::services::ExpenseService;
use crate::ui::theme::Theme;
use crate::ui::widgets::TableState;

pub fn expense_types() -> &'static [ExpenseType] {
    &[
        ExpenseType::Fixed,
        ExpenseType::Variable,
        ExpenseType::OneTime,
    ]
}

pub fn expense_categories() -> &'static [ExpenseCategory] {
    &[
        ExpenseCategory::Housing,
        ExpenseCategory::Transportation,
        ExpenseCategory::Food,
        ExpenseCategory::Healthcare,
        ExpenseCategory::Entertainment,
        ExpenseCategory::Utilities,
        ExpenseCategory::Insurance,
        ExpenseCategory::Subscriptions,
        ExpenseCategory::PersonalCare,
        ExpenseCategory::Education,
        ExpenseCategory::Savings,
        ExpenseCategory::Debt,
    ]
}

/// Form state for adding/editing expense
#[derive(Debug, Clone)]
pub struct ExpenseFormState {
    pub is_open: bool,
    pub is_edit: bool,
    pub active_field: usize, // 0: var_name, 1: display_name, 2: amount, 3: type, 4: freq, 5: category
    pub var_name: InputState,
    pub display_name: InputState,
    pub amount: InputState,
    pub type_idx: usize,
    pub frequency_idx: usize,
    pub category_idx: usize,
    pub error: Option<String>,
    pub edit_id: Option<String>,
}

impl Default for ExpenseFormState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_edit: false,
            active_field: 0,
            var_name: InputState::new(),
            display_name: InputState::new(),
            amount: InputState::new(),
            type_idx: 0,      // Fixed
            frequency_idx: 3, // Monthly
            category_idx: 0,  // Housing
            error: None,
            edit_id: None,
        }
    }
}

/// Expenses view state
#[derive(Debug, Clone)]
pub struct ExpensesState {
    /// Table state for navigation
    pub table_state: TableState,
    /// Cached expenses
    pub expenses: Vec<Expense>,
    /// Total monthly expenses
    pub total_monthly: Decimal,
    /// Total fixed expenses
    pub total_fixed: Decimal,
    /// Total variable expenses
    pub total_variable: Decimal,
    /// Form state
    pub form: ExpenseFormState,
}

impl Default for ExpensesState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpensesState {
    /// Create a new expenses view state
    pub fn new() -> Self {
        Self {
            table_state: TableState::new(),
            expenses: Vec::new(),
            total_monthly: Decimal::ZERO,
            total_fixed: Decimal::ZERO,
            total_variable: Decimal::ZERO,
            form: ExpenseFormState::default(),
        }
    }

    /// Refresh expense data from database
    pub fn refresh(&mut self, db: &Database) {
        let service = ExpenseService::new(db);
        self.expenses = service.get_all().unwrap_or_default();
        self.total_monthly = service.total_monthly_expenses().unwrap_or(Decimal::ZERO);
        self.total_fixed = service.total_fixed_expenses().unwrap_or(Decimal::ZERO);
        self.total_variable = service.total_variable_expenses().unwrap_or(Decimal::ZERO);
        self.table_state.set_total(self.expenses.len());
    }

    /// Get currently selected expense
    pub fn selected(&self) -> Option<&Expense> {
        self.expenses.get(self.table_state.selected)
    }

    /// Navigation
    pub fn next(&mut self) {
        self.table_state.next();
    }
    pub fn previous(&mut self) {
        self.table_state.previous();
    }
    pub fn first(&mut self) {
        self.table_state.first();
    }
    pub fn last(&mut self) {
        self.table_state.last();
    }
}

/// Expenses view widget
pub struct ExpensesView<'a> {
    state: &'a ExpensesState,
    theme: &'a Theme,
}

impl<'a> ExpensesView<'a> {
    pub fn new(state: &'a ExpensesState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Render the header summary
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Expenses Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(inner);

        // Total Monthly
        let total = Paragraph::new(Line::from(vec![
            Span::styled("Total: ", Style::default().fg(self.theme.colors.text_muted)),
            Span::styled(
                format!("${:.2}", self.state.total_monthly),
                Style::default()
                    .fg(self.theme.colors.error)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center);
        total.render(chunks[0], buf);

        // Fixed expenses
        let fixed = Paragraph::new(Line::from(vec![
            Span::styled("Fixed: ", Style::default().fg(self.theme.colors.text_muted)),
            Span::styled(
                format!("${:.2}", self.state.total_fixed),
                Style::default().fg(self.theme.colors.warning),
            ),
        ]))
        .alignment(Alignment::Center);
        fixed.render(chunks[1], buf);

        // Variable expenses
        let variable = Paragraph::new(Line::from(vec![
            Span::styled(
                "Variable: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", self.state.total_variable),
                Style::default().fg(self.theme.colors.info),
            ),
        ]))
        .alignment(Alignment::Center);
        variable.render(chunks[2], buf);

        // Active count
        let active_count = self.state.expenses.iter().filter(|e| e.is_active).count();
        let sources = Paragraph::new(Line::from(vec![
            Span::styled(
                "Active: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("{}/{}", active_count, self.state.expenses.len()),
                Style::default().fg(self.theme.colors.text_primary),
            ),
        ]))
        .alignment(Alignment::Center);
        sources.render(chunks[3], buf);
    }

    /// Render the expenses list
    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Expenses ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.state.expenses.is_empty() {
            let empty = Paragraph::new("No expenses. Press [a] to add one.")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            empty.render(inner, buf);
            return;
        }

        // Define layout
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(3),  // Cursor
                Constraint::Fill(1),    // Variable
                Constraint::Fill(2),    // Name
                Constraint::Length(12), // Category
                Constraint::Length(8),  // Type
                Constraint::Length(12), // Monthly
                Constraint::Length(4),  // Active
            ]);

        // Render header row
        if inner.height > 0 {
            let header_area = Rect {
                x: inner.x,
                y: inner.y,
                width: inner.width,
                height: 1,
            };

            let cols = layout.split(header_area);
            let header_style = Style::default()
                .fg(self.theme.colors.text_secondary)
                .add_modifier(Modifier::BOLD);

            let headers = ["", "Variable", "Name", "Category", "Type", "Monthly", "Act"];
            let alignments = [
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Center,
            ];

            for (i, col) in cols.iter().enumerate() {
                if i < headers.len() {
                    Paragraph::new(headers[i])
                        .style(header_style)
                        .alignment(alignments[i])
                        .render(*col, buf);
                }
            }
        }

        // Render expense rows
        let rows_area = Rect {
            y: inner.y + 1,
            height: inner.height.saturating_sub(1),
            ..inner
        };

        let visible_rows = rows_area.height as usize;
        let start = self.state.table_state.offset;

        for (i, expense) in self
            .state
            .expenses
            .iter()
            .enumerate()
            .skip(start)
            .take(visible_rows)
        {
            let y = rows_area.y + (i - start) as u16;
            let row_area = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };

            let is_selected = i == self.state.table_state.selected;
            let cols = layout.split(row_area);

            let style = if is_selected {
                Style::default()
                    .fg(self.theme.colors.text_primary)
                    .bg(self.theme.colors.surface_variant)
            } else if expense.is_active {
                Style::default().fg(self.theme.colors.text_primary)
            } else {
                Style::default().fg(self.theme.colors.text_muted)
            };

            let essential_marker = if expense.is_essential { "!" } else { "" };

            // Background for selection
            if is_selected {
                buf.set_style(row_area, style);
            }

            // 0. Cursor
            Paragraph::new(if is_selected { " ▶" } else { "" })
                .style(style)
                .render(cols[0], buf);

            // 1. Variable
            Paragraph::new(format!("${}", expense.variable_name))
                .style(style)
                .alignment(Alignment::Left)
                .render(cols[1], buf);

            // 2. Name
            Paragraph::new(format!("{}{}", expense.display_name, essential_marker))
                .style(style)
                .alignment(Alignment::Left)
                .render(cols[2], buf);

            // 3. Category
            Paragraph::new(format_category(&expense.category))
                .style(style)
                .alignment(Alignment::Left)
                .render(cols[3], buf);

            // 4. Type
            Paragraph::new(format_type(&expense.expense_type))
                .style(style)
                .alignment(Alignment::Left)
                .render(cols[4], buf);

            // 5. Monthly
            Paragraph::new(format!("${:.2}", expense.monthly_amount()))
                .style(style)
                .alignment(Alignment::Right)
                .render(cols[5], buf);

            // 6. Active
            Paragraph::new(if expense.is_active { "●" } else { "○" })
                .style(style)
                .alignment(Alignment::Center)
                .render(cols[6], buf);
        }
    }

    /// Render the help footer
    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help = Paragraph::new(Line::from(vec![
            Span::styled("[a]", Style::default().fg(self.theme.colors.primary)),
            Span::raw("dd "),
            Span::styled("[e]", Style::default().fg(self.theme.colors.primary)),
            Span::raw("dit "),
            Span::styled("[d]", Style::default().fg(self.theme.colors.primary)),
            Span::raw("elete "),
            Span::styled("[t]", Style::default().fg(self.theme.colors.primary)),
            Span::raw("oggle "),
            Span::styled("[!]", Style::default().fg(self.theme.colors.primary)),
            Span::raw(" essential "),
            Span::styled("[j/k]", Style::default().fg(self.theme.colors.primary)),
            Span::raw(" nav"),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(self.theme.colors.text_muted));

        help.render(area, buf);
    }

    /// Render the add/edit form popup
    fn render_form(&self, area: Rect, buf: &mut Buffer) {
        let popup_width = 50;
        let popup_height = 20;
        let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        Clear.render(popup_area, buf);

        let title = if self.state.form.is_edit {
            " Edit Expense "
        } else {
            " Add Expense "
        };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.accent));
        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let active_style = Style::default().fg(self.theme.colors.accent);
        let normal_style = Style::default().fg(self.theme.colors.text_primary);

        // Var Name
        buf.set_string(
            inner.x + 2,
            inner.y + 1,
            "Variable:",
            if self.state.form.active_field == 0 {
                active_style
            } else {
                normal_style
            },
        );
        let var_rect = Rect::new(inner.x + 12, inner.y + 1, inner.width - 14, 1);
        let var_input = TextInput::new(&self.state.form.var_name, self.theme)
            .placeholder("rent")
            .block(Block::default());
        if self.state.form.active_field == 0 {
            let mut state = self.state.form.var_name.clone();
            state.focus();
            TextInput::new(&state, self.theme)
                .placeholder("rent")
                .block(Block::default())
                .render(var_rect, buf);
        } else {
            var_input.render(var_rect, buf);
        }

        // Display Name
        buf.set_string(
            inner.x + 2,
            inner.y + 4,
            "Name:",
            if self.state.form.active_field == 1 {
                active_style
            } else {
                normal_style
            },
        );
        let name_rect = Rect::new(inner.x + 12, inner.y + 4, inner.width - 14, 1);
        let name_input = TextInput::new(&self.state.form.display_name, self.theme)
            .placeholder("Apartment Rent")
            .block(Block::default());
        if self.state.form.active_field == 1 {
            let mut state = self.state.form.display_name.clone();
            state.focus();
            TextInput::new(&state, self.theme)
                .placeholder("Apartment Rent")
                .block(Block::default())
                .render(name_rect, buf);
        } else {
            name_input.render(name_rect, buf);
        }

        // Amount
        buf.set_string(
            inner.x + 2,
            inner.y + 7,
            "Amount:",
            if self.state.form.active_field == 2 {
                active_style
            } else {
                normal_style
            },
        );
        let amount_rect = Rect::new(inner.x + 12, inner.y + 7, inner.width - 14, 1);
        let amount_input = TextInput::new(&self.state.form.amount, self.theme)
            .placeholder("1000.00")
            .block(Block::default());
        if self.state.form.active_field == 2 {
            let mut state = self.state.form.amount.clone();
            state.focus();
            TextInput::new(&state, self.theme)
                .placeholder("1000.00")
                .block(Block::default())
                .render(amount_rect, buf);
        } else {
            amount_input.render(amount_rect, buf);
        }

        // Type
        buf.set_string(
            inner.x + 2,
            inner.y + 10,
            "Type:",
            if self.state.form.active_field == 3 {
                active_style
            } else {
                normal_style
            },
        );
        let type_str = format_type(&expense_types()[self.state.form.type_idx]);
        buf.set_string(
            inner.x + 12,
            inner.y + 10,
            format!("< {} >", type_str),
            if self.state.form.active_field == 3 {
                active_style
            } else {
                normal_style
            },
        );

        // Frequency
        buf.set_string(
            inner.x + 2,
            inner.y + 12,
            "Freq:",
            if self.state.form.active_field == 4 {
                active_style
            } else {
                normal_style
            },
        );
        let freq_str = crate::ui::views::income::format_frequency(
            &crate::ui::views::income::frequencies()[self.state.form.frequency_idx],
        );
        buf.set_string(
            inner.x + 12,
            inner.y + 12,
            format!("< {} >", freq_str),
            if self.state.form.active_field == 4 {
                active_style
            } else {
                normal_style
            },
        );

        // Category
        buf.set_string(
            inner.x + 2,
            inner.y + 14,
            "Category:",
            if self.state.form.active_field == 5 {
                active_style
            } else {
                normal_style
            },
        );
        let cat_str = format_category(&expense_categories()[self.state.form.category_idx]);
        buf.set_string(
            inner.x + 12,
            inner.y + 14,
            format!("< {} >", cat_str),
            if self.state.form.active_field == 5 {
                active_style
            } else {
                normal_style
            },
        );

        // Error message
        if let Some(err) = &self.state.form.error {
            buf.set_string(
                inner.x + 2,
                inner.y + 16,
                err,
                Style::default().fg(self.theme.colors.error),
            );
        }

        // Footer
        buf.set_string(
            inner.x + 2,
            inner.y + 18,
            "Tab/Shift+Tab: move | Enter: save | Esc: cancel",
            Style::default().fg(self.theme.colors.text_muted),
        );
    }
}

impl Widget for ExpensesView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header summary
                Constraint::Min(5),    // List
                Constraint::Length(1), // Help
            ])
            .split(area);

        self.render_header(chunks[0], buf);
        self.render_list(chunks[1], buf);
        self.render_help(chunks[2], buf);

        if self.state.form.is_open {
            self.render_form(area, buf);
        }
    }
}

/// Format expense category for display
fn format_category(cat: &ExpenseCategory) -> String {
    match cat {
        ExpenseCategory::Housing => "Housing".to_string(),
        ExpenseCategory::Transportation => "Transport".to_string(),
        ExpenseCategory::Food => "Food".to_string(),
        ExpenseCategory::Healthcare => "Health".to_string(),
        ExpenseCategory::Entertainment => "Entertain".to_string(),
        ExpenseCategory::Utilities => "Utilities".to_string(),
        ExpenseCategory::Insurance => "Insurance".to_string(),
        ExpenseCategory::Subscriptions => "Subscript".to_string(),
        ExpenseCategory::PersonalCare => "Personal".to_string(),
        ExpenseCategory::Education => "Education".to_string(),
        ExpenseCategory::Savings => "Savings".to_string(),
        ExpenseCategory::Debt => "Debt".to_string(),
        ExpenseCategory::Custom(s) => truncate(s, 9),
    }
}

/// Format expense type for display
fn format_type(t: &ExpenseType) -> String {
    match t {
        ExpenseType::Fixed => "Fixed".to_string(),
        ExpenseType::Variable => "Var".to_string(),
        ExpenseType::OneTime => "Once".to_string(),
    }
}

/// Truncate string
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
