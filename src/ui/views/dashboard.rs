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
    widgets::{Block, Borders, Paragraph, Widget},
};
use rust_decimal::Decimal;

use crate::domain::transaction::{Transaction, TransactionType};
use crate::repository::Database;
use crate::services::{
    BudgetService, BudgetSummary, BudgetWarning, ChartService, ExpenseService, IncomeService,
    MonthSummary, TransactionService, WarningSeverity,
};
use crate::ui::theme::Theme;

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
        }
    }

    /// Refresh all dashboard data from the database
    pub fn refresh(&mut self, db: &Database) {
        let income_service = IncomeService::new(db);
        let expense_service = ExpenseService::new(db);
        let tx_service = TransactionService::new(db);
        let budget_service = BudgetService::new(db);
        let chart_service = ChartService::new(db);

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
}

impl<'a> Dashboard<'a> {
    /// Create a new dashboard widget
    pub fn new(state: &'a DashboardState, theme: &'a Theme) -> Self {
        Self { state, theme }
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
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .split(inner);

        // Net balance (big number)
        let balance_color = if self.state.net_balance >= Decimal::ZERO {
            self.theme.colors.success
        } else {
            self.theme.colors.error
        };

        let sign = if self.state.net_balance >= Decimal::ZERO {
            "+"
        } else {
            ""
        };
        let balance_text = format!("{}${:.2}", sign, self.state.net_balance.abs());

        let balance = Paragraph::new(Line::from(vec![Span::styled(
            balance_text,
            Style::default()
                .fg(balance_color)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center);
        balance.render(chunks[0], buf);

        // Income and Expenses side by side
        let detail_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let income_text = Paragraph::new(Line::from(vec![
            Span::styled(
                "Income: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", self.state.total_income),
                Style::default().fg(self.theme.colors.success),
            ),
        ]))
        .alignment(Alignment::Center);
        income_text.render(detail_chunks[0], buf);

        let expense_text = Paragraph::new(Line::from(vec![
            Span::styled(
                "Expenses: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", self.state.total_expenses),
                Style::default().fg(self.theme.colors.error),
            ),
        ]))
        .alignment(Alignment::Center);
        expense_text.render(detail_chunks[1], buf);

        // Savings rate
        if chunks[2].height >= 1 {
            let rate_color = if self.state.savings_rate >= Decimal::from(20) {
                self.theme.colors.success
            } else if self.state.savings_rate >= Decimal::from(10) {
                self.theme.colors.warning
            } else {
                self.theme.colors.error
            };

            let rate_text = Paragraph::new(Line::from(vec![
                Span::styled(
                    "Savings Rate: ",
                    Style::default().fg(self.theme.colors.text_muted),
                ),
                Span::styled(
                    format!("{:.1}%", self.state.savings_rate),
                    Style::default().fg(rate_color),
                ),
            ]))
            .alignment(Alignment::Center);
            rate_text.render(chunks[2], buf);
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
            let amount_str = format!(
                "{}${:.2}",
                if tx.transaction_type == TransactionType::Income {
                    "+"
                } else {
                    "-"
                },
                tx.amount.abs()
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
                format!("${:.2}", self.state.fixed_expenses),
            ),
            (
                "Variable Expenses",
                format!("${:.2}", self.state.variable_expenses),
            ),
            (
                "Projected Balance",
                format!("${:.2}", self.state.projected_balance),
            ),
            (
                "Transactions",
                self.state
                    .month_summary
                    .as_ref()
                    .map(|s| (s.income_count + s.expense_count).to_string())
                    .unwrap_or_else(|| "0".to_string()),
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
    }
}
