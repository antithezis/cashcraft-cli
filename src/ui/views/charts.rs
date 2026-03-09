//! Charts view
//!
//! Data visualizations including:
//! - Income vs Expenses trends
//! - Category breakdown
//! - Savings trend
//! - Daily spending

use chrono::{Datelike, Local};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::repository::Database;
use crate::services::{CategoryBreakdown, ChartService, IncomeExpensePoint, SavingsPoint};
use crate::ui::theme::Theme;
use crate::ui::widgets::{BarChart, DataPoint};

/// Chart type selection
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ChartType {
    #[default]
    IncomeVsExpenses,
    CategoryBreakdown,
    SavingsTrend,
    DailySpending,
}

impl ChartType {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            ChartType::IncomeVsExpenses => "Income vs Expenses",
            ChartType::CategoryBreakdown => "Category Breakdown",
            ChartType::SavingsTrend => "Savings Trend",
            ChartType::DailySpending => "Daily Spending",
        }
    }

    /// Get all chart types
    pub fn all() -> &'static [ChartType] {
        &[
            ChartType::IncomeVsExpenses,
            ChartType::CategoryBreakdown,
            ChartType::SavingsTrend,
            ChartType::DailySpending,
        ]
    }
}

/// State for the charts view
#[derive(Debug, Clone)]
pub struct ChartsState {
    /// Selected chart type
    pub chart_type: ChartType,
    /// Current viewing year
    pub year: i32,
    /// Current viewing month (1-12)
    pub month: u32,
    /// Number of months for trend charts
    pub months_back: usize,
    /// Income vs expenses data
    pub income_expense_data: Vec<IncomeExpensePoint>,
    /// Category breakdown data
    pub category_breakdown: Vec<CategoryBreakdown>,
    /// Savings trend data
    pub savings_data: Vec<SavingsPoint>,
}

impl Default for ChartsState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartsState {
    /// Create new charts state
    pub fn new() -> Self {
        let now = Local::now().date_naive();
        Self {
            chart_type: ChartType::IncomeVsExpenses,
            year: now.year(),
            month: now.month(),
            months_back: 6,
            income_expense_data: Vec::new(),
            category_breakdown: Vec::new(),
            savings_data: Vec::new(),
        }
    }

    /// Refresh chart data from database
    pub fn refresh(&mut self, db: &Database) {
        let service = ChartService::new(db);

        self.income_expense_data = service
            .income_vs_expenses(self.months_back)
            .unwrap_or_default();

        self.category_breakdown = service
            .category_breakdown(self.year, self.month)
            .unwrap_or_default();

        self.savings_data = service.savings_trend(self.months_back).unwrap_or_default();
    }

    /// Select next chart type
    pub fn next_chart(&mut self) {
        self.chart_type = match self.chart_type {
            ChartType::IncomeVsExpenses => ChartType::CategoryBreakdown,
            ChartType::CategoryBreakdown => ChartType::SavingsTrend,
            ChartType::SavingsTrend => ChartType::DailySpending,
            ChartType::DailySpending => ChartType::IncomeVsExpenses,
        };
    }

    /// Select previous chart type
    pub fn prev_chart(&mut self) {
        self.chart_type = match self.chart_type {
            ChartType::IncomeVsExpenses => ChartType::DailySpending,
            ChartType::CategoryBreakdown => ChartType::IncomeVsExpenses,
            ChartType::SavingsTrend => ChartType::CategoryBreakdown,
            ChartType::DailySpending => ChartType::SavingsTrend,
        };
    }
}

/// Charts view widget
pub struct ChartsView<'a> {
    state: &'a ChartsState,
    theme: &'a Theme,
}

impl<'a> ChartsView<'a> {
    /// Create new charts view
    pub fn new(state: &'a ChartsState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Render chart type selector
    fn render_selector(&self, area: Rect, buf: &mut Buffer) {
        let mut spans = vec![Span::styled(
            "Chart: ",
            Style::default().fg(self.theme.colors.text_muted),
        )];

        for (i, chart_type) in ChartType::all().iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }

            let style = if *chart_type == self.state.chart_type {
                Style::default()
                    .fg(self.theme.colors.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.colors.text_muted)
            };

            let label = if *chart_type == self.state.chart_type {
                format!("[<] [{}] {} [>]", i + 1, chart_type.name())
            } else {
                format!("[{}] {}", i + 1, chart_type.name())
            };

            spans.push(Span::styled(label, style));
        }

        let selector = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        selector.render(area, buf);
    }

    /// Render income vs expenses chart
    fn render_income_vs_expenses(&self, area: Rect, buf: &mut Buffer) {
        if self.state.income_expense_data.is_empty() {
            let empty = Paragraph::new("No transaction data available")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            empty.render(area, buf);
            return;
        }

        // Create data points for income and expenses
        // IncomeExpensePoint has `date: NaiveDate`, use `.month()` and `.year()`
        let income_points: Vec<DataPoint> = self
            .state
            .income_expense_data
            .iter()
            .map(|p| {
                DataPoint::new(
                    format!("{}/{}", p.date.month(), p.date.year() % 100),
                    p.income.to_string().parse::<f64>().unwrap_or(0.0),
                )
            })
            .collect();

        let expense_points: Vec<DataPoint> = self
            .state
            .income_expense_data
            .iter()
            .map(|p| {
                DataPoint::new(
                    format!("{}/{}", p.date.month(), p.date.year() % 100),
                    p.expenses.to_string().parse::<f64>().unwrap_or(0.0),
                )
            })
            .collect();

        // Split area for two charts side by side
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Income chart
        let income_chart = BarChart::new(income_points, self.theme).title("Income");
        income_chart.render(chunks[0], buf);

        // Expenses chart
        let expense_chart = BarChart::new(expense_points, self.theme).title("Expenses");
        expense_chart.render(chunks[1], buf);
    }

    /// Render category breakdown chart
    fn render_category_breakdown(&self, area: Rect, buf: &mut Buffer) {
        if self.state.category_breakdown.is_empty() {
            let empty = Paragraph::new("No spending data for this month")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            empty.render(area, buf);
            return;
        }

        // Create data points for categories
        let points: Vec<DataPoint> = self
            .state
            .category_breakdown
            .iter()
            .take(10) // Top 10 categories
            .map(|c| {
                // Truncate category name for display
                let name = if c.category.len() > 8 {
                    format!("{}...", &c.category[..5])
                } else {
                    c.category.clone()
                };
                DataPoint::new(name, c.amount.to_string().parse::<f64>().unwrap_or(0.0))
            })
            .collect();

        let chart = BarChart::new(points, self.theme).title("Spending by Category");
        chart.render(area, buf);
    }

    /// Render savings trend chart
    fn render_savings_trend(&self, area: Rect, buf: &mut Buffer) {
        if self.state.savings_data.is_empty() {
            let empty = Paragraph::new("No savings data available")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            empty.render(area, buf);
            return;
        }

        // Create data points for savings
        // SavingsPoint has `date: NaiveDate` and `monthly_savings: Decimal`
        let points: Vec<DataPoint> = self
            .state
            .savings_data
            .iter()
            .map(|s| {
                DataPoint::new(
                    format!("{}/{}", s.date.month(), s.date.year() % 100),
                    s.monthly_savings.to_string().parse::<f64>().unwrap_or(0.0),
                )
            })
            .collect();

        let chart = BarChart::new(points, self.theme).title("Monthly Savings");
        chart.render(area, buf);
    }

    /// Render daily spending (placeholder)
    fn render_daily_spending(&self, area: Rect, buf: &mut Buffer) {
        let placeholder = Paragraph::new("Daily spending chart - coming soon")
            .style(Style::default().fg(self.theme.colors.text_muted))
            .alignment(Alignment::Center);
        placeholder.render(area, buf);
    }
}

impl Widget for ChartsView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Charts ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 5 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Chart selector
                Constraint::Min(4),    // Chart area
            ])
            .split(inner);

        // Render selector
        self.render_selector(chunks[0], buf);

        // Render selected chart
        match self.state.chart_type {
            ChartType::IncomeVsExpenses => self.render_income_vs_expenses(chunks[1], buf),
            ChartType::CategoryBreakdown => self.render_category_breakdown(chunks[1], buf),
            ChartType::SavingsTrend => self.render_savings_trend(chunks[1], buf),
            ChartType::DailySpending => self.render_daily_spending(chunks[1], buf),
        }
    }
}
