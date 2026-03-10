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
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
};

use crate::repository::Database;
use crate::services::{
    CategoryBreakdown, ChartService, DailySpendingPoint, IncomeExpensePoint, SavingsPoint,
};
use crate::ui::theme::Theme;
use crate::ui::widgets::{BarChart, DataPoint};

/// Chart type selection
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ChartType {
    #[default]
    IncomeVsExpenses,
    CategoryBreakdown,
    PieChart,
    SavingsTrend,
    DailySpending,
}

impl ChartType {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            ChartType::IncomeVsExpenses => "Income vs Expenses",
            ChartType::CategoryBreakdown => "Category Breakdown (Bar)",
            ChartType::PieChart => "Category Breakdown (Pie)",
            ChartType::SavingsTrend => "Savings Trend",
            ChartType::DailySpending => "Daily Spending",
        }
    }

    /// Get all chart types
    pub fn all() -> &'static [ChartType] {
        &[
            ChartType::IncomeVsExpenses,
            ChartType::CategoryBreakdown,
            ChartType::PieChart,
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
    /// Daily spending data
    pub daily_data: Vec<DailySpendingPoint>,
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
            daily_data: Vec::new(),
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

        self.daily_data = service
            .daily_spending(self.year, self.month)
            .unwrap_or_default();
    }

    /// Select next chart type (Cyclical: Main -> Breakdown -> Daily)
    pub fn next_chart(&mut self) {
        self.chart_type = match self.chart_type {
            ChartType::IncomeVsExpenses | ChartType::SavingsTrend => ChartType::CategoryBreakdown,
            ChartType::CategoryBreakdown | ChartType::PieChart => ChartType::DailySpending,
            ChartType::DailySpending => ChartType::IncomeVsExpenses,
        };
    }

    /// Select previous chart type (Cyclical: Main <- Breakdown <- Daily)
    pub fn prev_chart(&mut self) {
        self.chart_type = match self.chart_type {
            ChartType::IncomeVsExpenses | ChartType::SavingsTrend => ChartType::DailySpending,
            ChartType::CategoryBreakdown | ChartType::PieChart => ChartType::IncomeVsExpenses,
            ChartType::DailySpending => ChartType::CategoryBreakdown,
        };
    }

    /// Toggle view mode for the current chart category
    pub fn toggle_view_mode(&mut self) {
        match self.chart_type {
            ChartType::CategoryBreakdown => self.chart_type = ChartType::PieChart,
            ChartType::PieChart => self.chart_type = ChartType::CategoryBreakdown,
            _ => {}
        }
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

        let views = [
            (
                "Main",
                matches!(
                    self.state.chart_type,
                    ChartType::IncomeVsExpenses | ChartType::SavingsTrend
                ),
                "Income & Savings",
            ),
            (
                "Breakdown",
                matches!(
                    self.state.chart_type,
                    ChartType::CategoryBreakdown | ChartType::PieChart
                ),
                if self.state.chart_type == ChartType::PieChart {
                    "Category (Pie)"
                } else {
                    "Category (Bar)"
                },
            ),
            (
                "Daily",
                self.state.chart_type == ChartType::DailySpending,
                "Daily Spending",
            ),
        ];

        for (i, (name, active, desc)) in views.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }

            let style = if *active {
                Style::default()
                    .fg(self.theme.colors.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.colors.text_muted)
            };

            let label = if *active {
                format!("[<] [{}] {} [>]", i + 1, desc)
            } else {
                format!("[{}] {}", i + 1, name)
            };

            spans.push(Span::styled(label, style));
        }

        // Add hint for toggle if applicable
        if matches!(
            self.state.chart_type,
            ChartType::CategoryBreakdown | ChartType::PieChart
        ) {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                "(Space to toggle)",
                Style::default()
                    .fg(self.theme.colors.text_muted)
                    .add_modifier(Modifier::ITALIC),
            ));
        }

        let selector = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
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

        let mut income_data = Vec::new();
        let mut expense_data = Vec::new();
        let mut x_labels = Vec::new();
        let mut max_value = 0.0;

        for (i, point) in self.state.income_expense_data.iter().enumerate() {
            let income = point.income.to_string().parse::<f64>().unwrap_or(0.0);
            let expense = point.expenses.to_string().parse::<f64>().unwrap_or(0.0);

            income_data.push((i as f64, income));
            expense_data.push((i as f64, expense));

            if income > max_value {
                max_value = income;
            }
            if expense > max_value {
                max_value = expense;
            }

            x_labels.push(Span::raw(format!(
                "{}/{}",
                point.date.month(),
                point.date.year() % 100
            )));
        }

        let y_bound = if max_value == 0.0 {
            10.0
        } else {
            max_value * 1.1
        };

        let datasets = vec![
            Dataset::default()
                .name("Income")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&income_data),
            Dataset::default()
                .name("Expenses")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Red))
                .data(&expense_data),
        ];

        let x_axis = Axis::default()
            .title(Span::styled(
                "Month",
                Style::default().fg(self.theme.colors.text_muted),
            ))
            .style(Style::default().fg(self.theme.colors.text_muted))
            .bounds([0.0, (income_data.len().saturating_sub(1)) as f64])
            .labels(x_labels);

        let y_axis = Axis::default()
            .title(Span::styled(
                "Amount",
                Style::default().fg(self.theme.colors.text_muted),
            ))
            .style(Style::default().fg(self.theme.colors.text_muted))
            .bounds([0.0, y_bound])
            .labels(vec![
                Span::raw("0"),
                Span::raw(format!("{:.0}", y_bound / 2.0)),
                Span::raw(format!("{:.0}", y_bound)),
            ]);

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(" Income (Green) vs Expenses (Red) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.colors.border)),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        chart.render(area, buf);
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

    /// Render pie chart for category breakdown
    fn render_pie_chart(&self, area: Rect, buf: &mut Buffer) {
        if self.state.category_breakdown.is_empty() {
            let empty = Paragraph::new("No spending data for this month")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            empty.render(area, buf);
            return;
        }

        let total: f64 = self
            .state
            .category_breakdown
            .iter()
            .map(|c| c.amount.to_string().parse::<f64>().unwrap_or(0.0))
            .sum();

        if total == 0.0 {
            return;
        }

        let colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::White,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
        ];

        let block = Block::default()
            .title(" Category Distribution ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        // Draw stacked bar
        let bar_width = inner.width as usize;
        let mut spans = Vec::new();
        let mut current_pos = 0;

        for (i, c) in self.state.category_breakdown.iter().enumerate() {
            let amount = c.amount.to_string().parse::<f64>().unwrap_or(0.0);
            let percent = amount / total;
            let width = (percent * bar_width as f64).round() as usize;

            // Ensure at least 1 char if percentage > 0.01 and fits
            let width = if width == 0 && percent > 0.01 {
                1
            } else {
                width
            };

            // Avoid overflow
            let width = if current_pos + width > bar_width {
                bar_width.saturating_sub(current_pos)
            } else {
                width
            };

            if width > 0 {
                let color = colors[i % colors.len()];
                // Use block char for bar
                let bar_char = "█".repeat(width);
                spans.push(Span::styled(bar_char, Style::default().fg(color)));
                current_pos += width;
            }
        }

        // Fill remaining if any due to rounding
        if current_pos < bar_width {
            spans.push(Span::raw(" ".repeat(bar_width - current_pos)));
        }

        // Render bar
        let bar_line = Line::from(spans);
        buf.set_line(inner.x, inner.y + 1, &bar_line, inner.width);

        // Render Legend
        let legend_start_y = inner.y + 3;
        let legend_height = inner.height.saturating_sub(3);

        for (i, c) in self
            .state
            .category_breakdown
            .iter()
            .enumerate()
            .take(legend_height as usize)
        {
            let color = colors[i % colors.len()];
            let amount = c.amount.to_string().parse::<f64>().unwrap_or(0.0);
            let percent = (amount / total) * 100.0;

            let line = Line::from(vec![
                Span::styled("■ ", Style::default().fg(color)),
                Span::raw(format!("{}: ${:.2} ({:.1}%)", c.category, amount, percent)),
            ]);

            buf.set_line(inner.x, legend_start_y + i as u16, &line, inner.width);
        }
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

        let mut data = Vec::new();
        let mut x_labels = Vec::new();
        let mut max_value = 0.0;
        let mut min_value = 0.0;

        for (i, s) in self.state.savings_data.iter().enumerate() {
            let savings = s.monthly_savings.to_string().parse::<f64>().unwrap_or(0.0);
            data.push((i as f64, savings));

            if savings > max_value {
                max_value = savings;
            }
            if savings < min_value {
                min_value = savings;
            }

            x_labels.push(Span::raw(format!(
                "{}/{}",
                s.date.month(),
                s.date.year() % 100
            )));
        }

        let y_upper = if max_value == 0.0 {
            100.0
        } else {
            max_value * 1.1
        };

        let y_lower = if min_value == 0.0 {
            0.0
        } else {
            min_value * 1.1
        };

        let datasets = vec![Dataset::default()
            .name("Savings")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&data)];

        let x_axis = Axis::default()
            .title(Span::styled(
                "Month",
                Style::default().fg(self.theme.colors.text_muted),
            ))
            .style(Style::default().fg(self.theme.colors.text_muted))
            .bounds([0.0, (data.len().saturating_sub(1)) as f64])
            .labels(x_labels);

        let y_axis = Axis::default()
            .title(Span::styled(
                "Amount",
                Style::default().fg(self.theme.colors.text_muted),
            ))
            .style(Style::default().fg(self.theme.colors.text_muted))
            .bounds([y_lower, y_upper])
            .labels(vec![
                Span::raw(format!("{:.0}", y_lower)),
                Span::raw("0"),
                Span::raw(format!("{:.0}", y_upper)),
            ]);

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(" Monthly Savings Trend ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.colors.border)),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        chart.render(area, buf);
    }

    /// Render daily spending chart
    fn render_daily_spending(&self, area: Rect, buf: &mut Buffer) {
        if self.state.daily_data.is_empty() {
            let empty = Paragraph::new("No daily spending data for this month")
                .style(Style::default().fg(self.theme.colors.text_muted))
                .alignment(Alignment::Center);
            empty.render(area, buf);
            return;
        }

        let mut data = Vec::new();
        let mut max_value = 0.0;

        for d in &self.state.daily_data {
            let day = d.date.day() as f64;
            let amount = d.amount.to_string().parse::<f64>().unwrap_or(0.0);
            data.push((day, amount));

            if amount > max_value {
                max_value = amount;
            }
        }

        // Sort by day to ensure line connects correctly
        data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut x_labels = Vec::new();
        // Generate labels for every 5 days
        for i in (1..=31).step_by(5) {
            x_labels.push(Span::raw(format!("{}", i)));
        }

        let y_bound = if max_value == 0.0 {
            100.0
        } else {
            max_value * 1.1
        };

        let datasets = vec![Dataset::default()
            .name("Spending")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&data)];

        let x_axis = Axis::default()
            .title(Span::styled(
                "Day",
                Style::default().fg(self.theme.colors.text_muted),
            ))
            .style(Style::default().fg(self.theme.colors.text_muted))
            .bounds([1.0, 31.0])
            .labels(x_labels);

        let y_axis = Axis::default()
            .title(Span::styled(
                "Amount",
                Style::default().fg(self.theme.colors.text_muted),
            ))
            .style(Style::default().fg(self.theme.colors.text_muted))
            .bounds([0.0, y_bound])
            .labels(vec![
                Span::raw("0"),
                Span::raw(format!("{:.0}", y_bound / 2.0)),
                Span::raw(format!("{:.0}", y_bound)),
            ]);

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(" Daily Spending ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.colors.border)),
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        chart.render(area, buf);
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
            ChartType::IncomeVsExpenses | ChartType::SavingsTrend => {
                let sub_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(chunks[1]);

                self.render_income_vs_expenses(sub_chunks[0], buf);
                self.render_savings_trend(sub_chunks[1], buf);
            }
            ChartType::CategoryBreakdown => self.render_category_breakdown(chunks[1], buf),
            ChartType::PieChart => self.render_pie_chart(chunks[1], buf),
            ChartType::DailySpending => self.render_daily_spending(chunks[1], buf),
        }
    }
}
