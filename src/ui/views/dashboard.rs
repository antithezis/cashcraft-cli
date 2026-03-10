//! Dashboard view
//!
//! Main overview screen showing:
//! - Net balance summary (total income - total expenses)
//! - Recent transactions list
//! - Budget status overview
//! - Quick stats

use chrono::{Datelike, Local};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use rust_decimal::Decimal;

use crate::config::Settings;
use crate::ui::widgets::{InputState, TextInput};
use crate::utils::currency::format_currency;

use crate::domain::transaction::{Transaction, TransactionType};
use crate::repository::Database;
use crate::repository::{BalanceRepository, TransactionRepository};
use crate::services::{
    BalanceService, BudgetService, BudgetSummary, BudgetWarning, ChartService, ExpenseService,
    IncomeService, MonthSummary, TransactionService, WarningSeverity,
};
use crate::ui::theme::Theme;

/// Form state for setting opening balance
#[derive(Debug, Clone, Default)]
pub struct OpeningBalanceFormState {
    pub is_open: bool,
    pub amount: InputState,
    pub error: Option<String>,
}

/// Cached state for the dashboard view
#[derive(Debug, Clone)]
pub struct DashboardState {
    /// Total monthly income (from defined sources)
    pub total_income: Decimal,
    /// Total monthly expenses (from defined sources)
    pub total_expenses: Decimal,
    /// Net balance (income - expenses)
    pub net_balance: Decimal,
    /// Projected balance from chart service
    pub projected_balance: Decimal,
    /// Recent transactions
    pub recent_transactions: Vec<Transaction>,
    /// Current month summary
    pub month_summary: Option<MonthSummary>,
    /// Budget summary for current month
    pub budget_summary: Option<BudgetSummary>,
    /// Budget warnings
    pub budget_warnings: Vec<BudgetWarning>,
    /// Current year
    pub year: i32,
    /// Current month
    pub month: u32,
    /// Fixed expenses total
    pub fixed_expenses: Decimal,
    /// Variable expenses total
    pub variable_expenses: Decimal,
    /// Savings rate percentage
    pub savings_rate: Decimal,
    /// Monthly opening balance
    pub opening_balance: Decimal,
    /// Opening balance form
    pub form: OpeningBalanceFormState,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardState {
    /// Create a new empty dashboard state
    pub fn new() -> Self {
        let now = Local::now().date_naive();
        Self {
            total_income: Decimal::ZERO,
            total_expenses: Decimal::ZERO,
            net_balance: Decimal::ZERO,
            projected_balance: Decimal::ZERO,
            recent_transactions: Vec::new(),
            month_summary: None,
            budget_summary: None,
            budget_warnings: Vec::new(),
            year: now.year(),
            month: now.month(),
            fixed_expenses: Decimal::ZERO,
            variable_expenses: Decimal::ZERO,
            savings_rate: Decimal::ZERO,
            opening_balance: Decimal::ZERO,
            form: OpeningBalanceFormState::default(),
        }
    }

    /// Refresh all dashboard data from the database
    pub fn refresh(&mut self, db: &Database) {
        let income_service = IncomeService::new(db);
        let expense_service = ExpenseService::new(db);
        let tx_service = TransactionService::new(db);
        let budget_service = BudgetService::new(db);
        let chart_service = ChartService::new(db);
        let balance_repo = BalanceRepository::new(db);
        let tx_repo = TransactionRepository::new(db);
        let balance_service = BalanceService::new(&balance_repo, &tx_repo);

        // Get opening balance
        self.opening_balance = balance_service
            .get_opening_balance(self.year, self.month)
            .unwrap_or(Decimal::ZERO);

        // Get totals from defined sources
        self.total_income = income_service
            .total_monthly_income()
            .unwrap_or(Decimal::ZERO);
        self.total_expenses = expense_service
            .total_monthly_expenses()
            .unwrap_or(Decimal::ZERO);
        self.net_balance = self.total_income - self.total_expenses;

        // Get expense breakdown
        self.fixed_expenses = expense_service
            .total_fixed_expenses()
            .unwrap_or(Decimal::ZERO);
        self.variable_expenses = expense_service
            .total_variable_expenses()
            .unwrap_or(Decimal::ZERO);

        // Calculate savings rate
        if self.total_income > Decimal::ZERO {
            self.savings_rate = (self.net_balance / self.total_income) * Decimal::from(100);
        } else {
            self.savings_rate = Decimal::ZERO;
        }

        // Get projected balance (returns tuple: income, expenses, balance)
        self.projected_balance = chart_service
            .projected_monthly_balance()
            .map(|(_, _, balance)| balance)
            .unwrap_or(Decimal::ZERO);

        // Get recent transactions
        self.recent_transactions = tx_service.get_recent(8).unwrap_or_default();

        // Get current month summary
        self.month_summary = tx_service
            .calculate_monthly_summary(self.year, self.month)
            .ok();

        // Get budget info
        self.budget_summary = budget_service.get_month_summary(self.year, self.month).ok();
        self.budget_warnings = budget_service
            .check_warnings_for_month(self.year, self.month)
            .unwrap_or_default();
    }
}

/// Dashboard view widget
pub struct Dashboard<'a> {
    state: &'a DashboardState,
    theme: &'a Theme,
    settings: &'a Settings,
}

impl<'a> Dashboard<'a> {
    pub fn new(state: &'a DashboardState, theme: &'a Theme, settings: &'a Settings) -> Self {
        Self {
            state,
            theme,
            settings,
        }
    }

    /// Render the balance summary section
    fn render_balance(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Balance Overview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 4 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Opening Balance
                Constraint::Length(3), // Current Balance (Big)
                Constraint::Length(2), // Income / Expenses
                Constraint::Min(0),    // Savings Rate
            ])
            .split(inner);

        // Opening Balance
        let opening_text = format!(
            "Opening Balance: {}",
            format_currency(self.state.opening_balance, self.settings)
        );
        let opening = Paragraph::new(Line::from(vec![Span::styled(
            opening_text,
            Style::default().fg(self.theme.colors.text_muted),
        )]))
        .alignment(Alignment::Center);
        opening.render(chunks[0], buf);

        // Current Balance (Opening + Actual Net)
        let actual_net = self
            .state
            .month_summary
            .as_ref()
            .map(|s| s.net)
            .unwrap_or(Decimal::ZERO);
        let current_balance = self.state.opening_balance + actual_net;

        let balance_color = if current_balance >= Decimal::ZERO {
            self.theme.colors.success
        } else {
            self.theme.colors.error
        };

        let balance_text = if current_balance >= Decimal::ZERO {
            format!("+{}", format_currency(current_balance, self.settings))
        } else {
            format_currency(current_balance, self.settings)
        };

        let balance = Paragraph::new(Line::from(vec![Span::styled(
            balance_text,
            Style::default()
                .fg(balance_color)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        balance.render(chunks[1], buf);

        // Income and Expenses side by side (Budgeted/Projected)
        let detail_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        let income_text = Paragraph::new(Line::from(vec![
            Span::styled(
                "Proj. Income: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format_currency(self.state.total_income, self.settings),
                Style::default().fg(self.theme.colors.success),
            ),
        ]))
        .alignment(Alignment::Center);
        income_text.render(detail_chunks[0], buf);

        let expense_text = Paragraph::new(Line::from(vec![
            Span::styled(
                "Proj. Expenses: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format_currency(self.state.total_expenses, self.settings),
                Style::default().fg(self.theme.colors.error),
            ),
        ]))
        .alignment(Alignment::Center);
        expense_text.render(detail_chunks[1], buf);

        // Savings rate
        if chunks[3].height >= 1 {
            let rate_color = if self.state.savings_rate >= Decimal::from(20) {
                self.theme.colors.success
            } else if self.state.savings_rate >= Decimal::from(10) {
                self.theme.colors.warning
            } else {
                self.theme.colors.error
            };

            let rate_text = Paragraph::new(Line::from(vec![
                Span::styled(
                    "Proj. Savings Rate: ",
                    Style::default().fg(self.theme.colors.text_muted),
                ),
                Span::styled(
                    format!("{:.1}%", self.state.savings_rate),
                    Style::default().fg(rate_color),
                ),
            ]))
            .alignment(Alignment::Center);
            rate_text.render(chunks[3], buf);
        }
    }

    /// Render recent transactions section
    fn render_transactions(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Recent Transactions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.state.recent_transactions.is_empty() {
            let empty = Paragraph::new("No transactions yet")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            empty.render(inner, buf);
            return;
        }

        // Render transactions as lines
        let max_rows = inner.height as usize;
        for (i, tx) in self
            .state
            .recent_transactions
            .iter()
            .take(max_rows)
            .enumerate()
        {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            // Format: "Mar 09  Description            +$100.00"
            let date_str = tx.date.format("%b %d").to_string();
            let amount_val = format_currency(tx.amount.abs(), self.settings);
            let amount_str = format!(
                "{}{}",
                if tx.transaction_type == TransactionType::Income {
                    "+"
                } else {
                    "-"
                },
                amount_val
            );

            // Calculate description width
            let date_width = 7;
            let amount_width = amount_str.len() + 1;
            let desc_width = (inner.width as usize)
                .saturating_sub(date_width)
                .saturating_sub(amount_width);

            // Truncate description if needed
            let desc = if tx.description.len() > desc_width {
                format!("{}...", &tx.description[..desc_width.saturating_sub(3)])
            } else {
                tx.description.clone()
            };

            // Date
            buf.set_string(
                inner.x,
                y,
                &date_str,
                Style::default().fg(self.theme.colors.text_muted),
            );

            // Description
            buf.set_string(
                inner.x + date_width as u16,
                y,
                &desc,
                Style::default().fg(self.theme.colors.text_primary),
            );

            // Amount
            let amount_color = match tx.transaction_type {
                TransactionType::Income => self.theme.colors.success,
                TransactionType::Expense => self.theme.colors.error,
                TransactionType::Transfer => self.theme.colors.info,
            };
            let amount_x = inner.x + inner.width - amount_str.len() as u16;
            buf.set_string(amount_x, y, &amount_str, Style::default().fg(amount_color));
        }
    }

    /// Render budget overview section
    fn render_budget(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Budget Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.state.budget_warnings.is_empty() {
            let ok = Paragraph::new("All budgets on track")
                .style(Style::default().fg(self.theme.colors.success))
                .alignment(Alignment::Center);
            ok.render(inner, buf);
            return;
        }

        // Show warnings
        let max_rows = inner.height as usize;
        for (i, warning) in self.state.budget_warnings.iter().take(max_rows).enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            let (icon, color) = match warning.severity {
                WarningSeverity::Warning => ("!", self.theme.colors.warning),
                WarningSeverity::Critical => ("X", self.theme.colors.error),
            };

            // Truncate category if needed
            let max_cat_width = (inner.width as usize).saturating_sub(4);
            let cat = if warning.category.len() > max_cat_width {
                format!(
                    "{}...",
                    &warning.category[..max_cat_width.saturating_sub(3)]
                )
            } else {
                warning.category.clone()
            };

            buf.set_string(
                inner.x,
                y,
                format!("[{}] {}", icon, cat),
                Style::default().fg(color),
            );
        }
    }

    /// Render quick stats section
    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Quick Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        let stats = [
            (
                "Fixed Expenses",
                format_currency(self.state.fixed_expenses, self.settings),
            ),
            (
                "Variable Expenses",
                format_currency(self.state.variable_expenses, self.settings),
            ),
            (
                "Projected Balance",
                format_currency(self.state.projected_balance, self.settings),
            ),
        ];

        for (i, (label, value)) in stats.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            buf.set_string(
                inner.x,
                y,
                label,
                Style::default().fg(self.theme.colors.text_muted),
            );

            let value_x = inner.x + inner.width - value.len() as u16;
            buf.set_string(
                value_x,
                y,
                value,
                Style::default().fg(self.theme.colors.text_primary),
            );
        }
    }
    /// Render the opening balance form popup
    fn render_form(&self, area: Rect, buf: &mut Buffer) {
        let popup_width = 40;
        let popup_height = 10;
        let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        Clear.render(popup_area, buf);

        let block = Block::default()
            .title(" Set Opening Balance ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.accent));
        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let active_style = Style::default().fg(self.theme.colors.accent);

        // Amount Label
        buf.set_string(inner.x + 2, inner.y + 2, "Opening Balance:", active_style);

        // Amount Input
        let amount_rect = Rect::new(inner.x + 2, inner.y + 3, inner.width - 4, 1);
        let mut state = self.state.form.amount.clone();
        state.focus();
        TextInput::new(&state, self.theme)
            .placeholder("1000.00")
            .block(Block::default())
            .render(amount_rect, buf);

        // Error message
        if let Some(err) = &self.state.form.error {
            buf.set_string(
                inner.x + 2,
                inner.y + 5,
                err,
                Style::default().fg(self.theme.colors.error),
            );
        }

        // Footer
        buf.set_string(
            inner.x + 2,
            inner.y + 7,
            "Enter: save | Esc: cancel",
            Style::default().fg(self.theme.colors.text_muted),
        );
    }
}

impl Widget for Dashboard<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Main layout: top row (balance) and bottom row (3 columns)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(area);

        // Top: Balance overview (full width)
        self.render_balance(main_chunks[0], buf);

        // Bottom: 3 columns
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(main_chunks[1]);

        // Recent transactions (left, larger)
        self.render_transactions(bottom_chunks[0], buf);

        // Budget status (middle)
        self.render_budget(bottom_chunks[1], buf);

        // Quick stats (right)
        self.render_stats(bottom_chunks[2], buf);

        if self.state.form.is_open {
            self.render_form(area, buf);
        }
    }
}
