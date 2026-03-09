//! Budget view
//!
//! Budget planning and tracking with:
//! - Budget list with progress bars
//! - Category budgets
//! - Month navigation
//! - Warning indicators

use chrono::{Datelike, Local, NaiveDate};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use crate::ui::widgets::{InputState, TextInput};
use rust_decimal::Decimal;

use crate::domain::budget::Budget;
use crate::repository::Database;
use crate::services::{BudgetProgress, BudgetService, BudgetSummary};
use crate::ui::theme::Theme;
use crate::ui::widgets::{ProgressBar, TableState};

/// Form state for adding/editing a budget
#[derive(Debug, Clone)]
pub struct BudgetFormState {
    pub is_open: bool,
    pub is_edit: bool,
    pub active_field: usize, // 0: category, 1: amount
    pub category: InputState,
    pub amount: InputState,
    pub error: Option<String>,
    pub edit_id: Option<String>,
}

impl Default for BudgetFormState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_edit: false,
            active_field: 0,
            category: InputState::new(),
            amount: InputState::new(),
            error: None,
            edit_id: None,
        }
    }
}

/// State for the budget view
#[derive(Debug, Clone)]
pub struct BudgetState {
    /// Current viewing year
    pub year: i32,
    /// Current viewing month (1-12)
    pub month: u32,
    /// Table state for Vim navigation
    pub table_state: TableState,
    /// Budgets for current month
    pub budgets: Vec<Budget>,
    /// Budget progress for each budget
    pub progress: Vec<BudgetProgress>,
    /// Month summary
    pub summary: Option<BudgetSummary>,
    /// Form state
    pub form: BudgetFormState,
}

impl Default for BudgetState {
    fn default() -> Self {
        Self::new()
    }
}

impl BudgetState {
    /// Create new budget state for current month
    pub fn new() -> Self {
        let now = Local::now().date_naive();
        Self {
            year: now.year(),
            month: now.month(),
            table_state: TableState::new(),
            budgets: Vec::new(),
            progress: Vec::new(),
            summary: None,
            form: BudgetFormState::default(),
        }
    }

    /// Refresh budget data from database
    pub fn refresh(&mut self, db: &Database) {
        let service = BudgetService::new(db);

        self.budgets = service
            .get_month_budgets(self.year, self.month)
            .unwrap_or_default();

        // Calculate progress for all budgets at once
        self.progress = service
            .calculate_budget_progress(self.year, self.month)
            .unwrap_or_default();

        self.summary = service.get_month_summary(self.year, self.month).ok();
        self.table_state.set_total(self.budgets.len());
    }

    /// Navigate to next month
    pub fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
    }

    /// Navigate to previous month
    pub fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
    }

    /// Get selected budget
    pub fn selected_budget(&self) -> Option<&Budget> {
        self.budgets.get(self.table_state.selected)
    }

    /// Get progress for a budget by index
    fn get_progress(&self, index: usize) -> Option<&BudgetProgress> {
        // Find progress matching the budget at index
        let budget = self.budgets.get(index)?;
        self.progress.iter().find(|p| p.budget.id == budget.id)
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

/// Budget view widget
pub struct BudgetView<'a> {
    state: &'a BudgetState,
    theme: &'a Theme,
}

impl<'a> BudgetView<'a> {
    /// Create new budget view
    pub fn new(state: &'a BudgetState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Render the month header
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let month_name = NaiveDate::from_ymd_opt(self.state.year, self.state.month, 1)
            .map(|d| d.format("%B %Y").to_string())
            .unwrap_or_else(|| format!("{}/{}", self.state.month, self.state.year));

        let header = Paragraph::new(Line::from(vec![
            Span::styled("[ ", Style::default().fg(self.theme.colors.text_muted)),
            Span::styled(
                &month_name,
                Style::default()
                    .fg(self.theme.colors.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ]", Style::default().fg(self.theme.colors.text_muted)),
            Span::styled(
                "  ([ prev | ] next | c copy from prev)",
                Style::default().fg(self.theme.colors.text_muted),
            ),
        ]))
        .alignment(Alignment::Center);
        header.render(area, buf);
    }

    /// Render a budget row with progress bar
    fn render_row(
        &self,
        area: Rect,
        buf: &mut Buffer,
        budget: &Budget,
        progress: Option<&BudgetProgress>,
        selected: bool,
    ) {
        if area.height < 2 {
            // Single line mode
            self.render_row_compact(area, buf, budget, progress, selected);
        } else {
            // Two-line mode with progress bar
            self.render_row_expanded(area, buf, budget, progress, selected);
        }
    }

    /// Render compact single-line budget row
    fn render_row_compact(
        &self,
        area: Rect,
        buf: &mut Buffer,
        budget: &Budget,
        progress: Option<&BudgetProgress>,
        selected: bool,
    ) {
        let base_style = if selected {
            Style::default()
                .bg(self.theme.colors.surface)
                .fg(self.theme.colors.text_primary)
        } else {
            Style::default().fg(self.theme.colors.text_primary)
        };

        // Fill background if selected
        if selected {
            for x in area.x..area.x + area.width {
                buf.set_string(x, area.y, " ", base_style);
            }
        }

        // Category | Spent / Budget | Percentage
        let cat_w = 15;
        let amounts_w = 25;

        let mut x = area.x;

        // Category
        let cat = if budget.category.len() > cat_w - 1 {
            format!("{}...", &budget.category[..cat_w - 4])
        } else {
            budget.category.clone()
        };
        buf.set_string(x, area.y, &cat, base_style);
        x += cat_w as u16;

        // Spent / Budget
        let spent = progress.map(|p| p.spent).unwrap_or(Decimal::ZERO);
        let amounts = format!("${:.2} / ${:.2}", spent, budget.amount);
        buf.set_string(x, area.y, &amounts, base_style);
        x += amounts_w as u16;

        // Percentage with color (percentage is already f64)
        let pct = progress.map(|p| p.percentage).unwrap_or_else(|| {
            if budget.amount > Decimal::ZERO {
                let ratio: f64 = (Decimal::ZERO / budget.amount).try_into().unwrap_or(0.0);
                ratio * 100.0
            } else {
                0.0
            }
        });

        let pct_color = if pct >= 100.0 {
            self.theme.colors.error
        } else if pct >= 80.0 {
            self.theme.colors.warning
        } else {
            self.theme.colors.success
        };

        let pct_str = format!("{:.0}%", pct);
        buf.set_string(x, area.y, &pct_str, Style::default().fg(pct_color));
    }

    /// Render expanded two-line budget row with progress bar
    fn render_row_expanded(
        &self,
        area: Rect,
        buf: &mut Buffer,
        budget: &Budget,
        progress: Option<&BudgetProgress>,
        selected: bool,
    ) {
        let base_style = if selected {
            Style::default()
                .bg(self.theme.colors.surface)
                .fg(self.theme.colors.text_primary)
        } else {
            Style::default().fg(self.theme.colors.text_primary)
        };

        // Fill background if selected
        if selected {
            for y in area.y..area.y + area.height.min(2) {
                for x in area.x..area.x + area.width {
                    buf.set_string(x, y, " ", base_style);
                }
            }
        }

        // Line 1: Category and amounts
        let spent = progress.map(|p| p.spent).unwrap_or(Decimal::ZERO);
        let pct = progress.map(|p| p.percentage).unwrap_or_else(|| {
            if budget.amount > Decimal::ZERO {
                let ratio: f64 = (Decimal::ZERO / budget.amount).try_into().unwrap_or(0.0);
                ratio * 100.0
            } else {
                0.0
            }
        });

        let pct_color = if pct >= 100.0 {
            self.theme.colors.error
        } else if pct >= 80.0 {
            self.theme.colors.warning
        } else {
            self.theme.colors.success
        };

        let line1 = Line::from(vec![
            Span::styled(&budget.category, base_style.add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(format!("${:.2}", spent), Style::default().fg(pct_color)),
            Span::styled(" / ", Style::default().fg(self.theme.colors.text_muted)),
            Span::styled(format!("${:.2}", budget.amount), base_style),
            Span::raw("  "),
            Span::styled(format!("({:.0}%)", pct), Style::default().fg(pct_color)),
        ]);

        let para = Paragraph::new(line1);
        para.render(Rect::new(area.x, area.y, area.width, 1), buf);

        // Line 2: Progress bar
        if area.height >= 2 {
            let bar_area = Rect::new(area.x + 2, area.y + 1, area.width.saturating_sub(4), 1);
            let pct_normalized = (pct / 100.0).min(1.0);
            let bar = ProgressBar::new(pct_normalized, self.theme);
            bar.render(bar_area, buf);
        }
    }

    /// Render summary footer
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let summary = self.state.summary.as_ref();

        let total_budgeted = summary.map(|s| s.total_budgeted).unwrap_or(Decimal::ZERO);
        let total_spent = summary.map(|s| s.total_spent).unwrap_or(Decimal::ZERO);
        let remaining = total_budgeted - total_spent;

        let remaining_color = if remaining >= Decimal::ZERO {
            self.theme.colors.success
        } else {
            self.theme.colors.error
        };

        let footer = Paragraph::new(Line::from(vec![
            Span::styled(
                "Budgeted: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", total_budgeted),
                Style::default().fg(self.theme.colors.text_primary),
            ),
            Span::raw(" | "),
            Span::styled("Spent: ", Style::default().fg(self.theme.colors.text_muted)),
            Span::styled(
                format!("${:.2}", total_spent),
                Style::default().fg(self.theme.colors.error),
            ),
            Span::raw(" | "),
            Span::styled(
                "Remaining: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", remaining),
                Style::default().fg(remaining_color),
            ),
        ]))
        .alignment(Alignment::Center);
        footer.render(area, buf);
    }

    /// Render the add/edit form popup
    fn render_form(&self, area: Rect, buf: &mut Buffer) {
        let popup_width = 40;
        let popup_height = 12;
        let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        Clear.render(popup_area, buf);

        let title = if self.state.form.is_edit { " Edit Budget " } else { " Add Budget " };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.accent));
        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let active_style = Style::default().fg(self.theme.colors.accent);
        let normal_style = Style::default().fg(self.theme.colors.text_primary);

        // Category
        buf.set_string(inner.x + 2, inner.y + 1, "Category:", if self.state.form.active_field == 0 { active_style } else { normal_style });
        let cat_rect = Rect::new(inner.x + 12, inner.y + 1, inner.width - 14, 1);
        let cat_input = TextInput::new(&self.state.form.category, self.theme).placeholder("Groceries").block(Block::default());
        if self.state.form.active_field == 0 {
            let mut state = self.state.form.category.clone();
            state.focus();
            TextInput::new(&state, self.theme).placeholder("Groceries").block(Block::default()).render(cat_rect, buf);
        } else {
            cat_input.render(cat_rect, buf);
        }

        // Amount
        buf.set_string(inner.x + 2, inner.y + 4, "Amount:", if self.state.form.active_field == 1 { active_style } else { normal_style });
        let amount_rect = Rect::new(inner.x + 12, inner.y + 4, inner.width - 14, 1);
        let amount_input = TextInput::new(&self.state.form.amount, self.theme).placeholder("300.00").block(Block::default());
        if self.state.form.active_field == 1 {
            let mut state = self.state.form.amount.clone();
            state.focus();
            TextInput::new(&state, self.theme).placeholder("300.00").block(Block::default()).render(amount_rect, buf);
        } else {
            amount_input.render(amount_rect, buf);
        }

        // Error message
        if let Some(err) = &self.state.form.error {
            buf.set_string(inner.x + 2, inner.y + 6, err, Style::default().fg(self.theme.colors.error));
        }

        // Footer
        buf.set_string(inner.x + 2, inner.y + 8, "Tab/Shift+Tab: move | Enter: save | Esc: cancel", Style::default().fg(self.theme.colors.text_muted));
    }
}

impl Widget for BudgetView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Budget ")
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
                Constraint::Length(1), // Month header
                Constraint::Min(2),    // Budget list
                Constraint::Length(1), // Footer
            ])
            .split(inner);

        // Month header
        self.render_header(chunks[0], buf);

        if self.state.budgets.is_empty() {
            let empty = Paragraph::new(
                "No budgets set. Press 'a' to add one or 'c' to copy from previous month.",
            )
            .style(Style::default().fg(self.theme.colors.text_muted))
            .alignment(Alignment::Center);
            empty.render(chunks[1], buf);
        } else {
            // Determine row height (2 lines if space allows)
            let row_height = if chunks[1].height >= self.state.budgets.len() as u16 * 2 {
                2
            } else {
                1
            };

            let visible_rows = (chunks[1].height as usize) / row_height;
            let start = self.state.table_state.offset;
            let end = (start + visible_rows).min(self.state.budgets.len());

            for (row_idx, budget) in self
                .state
                .budgets
                .iter()
                .enumerate()
                .skip(start)
                .take(end - start)
            {
                let y = chunks[1].y + ((row_idx - start) * row_height) as u16;
                let row_area = Rect::new(chunks[1].x, y, chunks[1].width, row_height as u16);
                let selected = row_idx == self.state.table_state.selected;
                let progress = self.state.get_progress(row_idx);
                self.render_row(row_area, buf, budget, progress, selected);
            }
        }

        // Footer
        self.render_footer(chunks[2], buf);

        if self.state.form.is_open {
            self.render_form(area, buf);
        }
    }
}
