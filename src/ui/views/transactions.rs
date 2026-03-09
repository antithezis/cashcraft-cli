//! Transactions view
//!
//! Transaction history with:
//! - VimTable with Date, Description, Amount, Type, Category columns
//! - Month navigation with [ and ]
//! - Search/filter support
//! - CRUD operations (a, e, d)

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

use crate::domain::transaction::{Transaction, TransactionType};
use crate::repository::Database;
use crate::services::{MonthSummary, TransactionService};
use crate::ui::theme::Theme;
use crate::ui::widgets::TableState;

pub fn transaction_types() -> &'static [TransactionType] {
    &[
        TransactionType::Expense,
        TransactionType::Income,
        TransactionType::Transfer,
    ]
}

/// Form state for adding/editing transaction
#[derive(Debug, Clone)]
pub struct TransactionFormState {
    pub is_open: bool,
    pub is_edit: bool,
    pub active_field: usize, // 0: date, 1: description, 2: amount, 3: type, 4: category
    pub date: InputState,
    pub description: InputState,
    pub amount: InputState,
    pub type_idx: usize,
    pub category: InputState,
    pub error: Option<String>,
    pub edit_id: Option<String>,
}

impl Default for TransactionFormState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_edit: false,
            active_field: 0,
            date: InputState::new(), // Expected format YYYY-MM-DD
            description: InputState::new(),
            amount: InputState::new(),
            type_idx: 0, // Expense
            category: InputState::new(),
            error: None,
            edit_id: None,
        }
    }
}

/// State for the transactions view
#[derive(Debug, Clone)]
pub struct TransactionsState {
    /// Current viewing year
    pub year: i32,
    /// Current viewing month (1-12)
    pub month: u32,
    /// Table state for Vim navigation
    pub table_state: TableState,
    /// Cached transactions for current month
    pub transactions: Vec<Transaction>,
    /// Month summary
    pub month_summary: Option<MonthSummary>,
    /// Search query (when in search mode)
    pub search_query: String,
    /// Is search mode active
    pub searching: bool,
    /// Filtered transactions (when searching)
    pub filtered_indices: Option<Vec<usize>>,
    /// Form state
    pub form: TransactionFormState,
}

impl Default for TransactionsState {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionsState {
    /// Create new state for current month
    pub fn new() -> Self {
        let now = Local::now().date_naive();
        Self {
            year: now.year(),
            month: now.month(),
            table_state: TableState::new(),
            transactions: Vec::new(),
            month_summary: None,
            search_query: String::new(),
            searching: false,
            filtered_indices: None,
            form: TransactionFormState::default(),
        }
    }

    /// Refresh transactions from database
    pub fn refresh(&mut self, db: &Database) {
        let service = TransactionService::new(db);

        self.transactions = service
            .get_by_month(self.year, self.month)
            .unwrap_or_default();

        self.month_summary = service
            .calculate_monthly_summary(self.year, self.month)
            .ok();

        self.table_state.set_total(self.visible_count());
        self.filtered_indices = None;
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

    /// Get the currently selected transaction
    pub fn selected_transaction(&self) -> Option<&Transaction> {
        let index = if let Some(ref indices) = self.filtered_indices {
            indices.get(self.table_state.selected).copied()?
        } else {
            self.table_state.selected
        };
        self.transactions.get(index)
    }

    /// Get visible transaction count (filtered or all)
    fn visible_count(&self) -> usize {
        self.filtered_indices
            .as_ref()
            .map(|i| i.len())
            .unwrap_or(self.transactions.len())
    }

    /// Apply search filter
    pub fn apply_search(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = None;
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_indices = Some(
                self.transactions
                    .iter()
                    .enumerate()
                    .filter(|(_, tx)| {
                        tx.description.to_lowercase().contains(&query)
                            || tx.category.to_lowercase().contains(&query)
                    })
                    .map(|(i, _)| i)
                    .collect(),
            );
        }
        self.table_state.set_total(self.visible_count());
        self.table_state.select(0);
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.searching = false;
        self.filtered_indices = None;
        self.table_state.set_total(self.transactions.len());
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

/// Transactions view widget
pub struct TransactionsView<'a> {
    state: &'a TransactionsState,
    theme: &'a Theme,
}

impl<'a> TransactionsView<'a> {
    /// Create new transactions view
    pub fn new(state: &'a TransactionsState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Format transaction type as colored string
    fn type_style(&self, tx_type: &TransactionType) -> Style {
        match tx_type {
            TransactionType::Income => Style::default().fg(self.theme.colors.success),
            TransactionType::Expense => Style::default().fg(self.theme.colors.error),
            TransactionType::Transfer => Style::default().fg(self.theme.colors.info),
        }
    }

    /// Format amount with sign
    fn format_amount(&self, amount: Decimal, tx_type: &TransactionType) -> String {
        let sign = match tx_type {
            TransactionType::Income => "+",
            TransactionType::Expense => "-",
            TransactionType::Transfer => "",
        };
        format!("{}${:.2}", sign, amount.abs())
    }

    /// Render the header with month navigation
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
                "  ([ prev | ] next)",
                Style::default().fg(self.theme.colors.text_muted),
            ),
        ]))
        .alignment(Alignment::Center);
        header.render(area, buf);
    }

    /// Render the table header row
    fn render_table_header(&self, area: Rect, buf: &mut Buffer) {
        // Column widths: Date(10) | Description(flex) | Amount(12) | Type(8) | Category(12)
        let date_w = 10;
        let amount_w = 12;
        let type_w = 8;
        let cat_w = 12;
        let desc_w = (area.width as usize).saturating_sub(date_w + amount_w + type_w + cat_w + 4);

        let header_style = Style::default()
            .fg(self.theme.colors.text_primary)
            .add_modifier(Modifier::BOLD);

        let mut x = area.x;
        buf.set_string(x, area.y, "Date", header_style);
        x += date_w as u16;
        buf.set_string(x, area.y, "Description", header_style);
        x += desc_w as u16;
        buf.set_string(x, area.y, "Amount", header_style);
        x += amount_w as u16;
        buf.set_string(x, area.y, "Type", header_style);
        x += type_w as u16;
        buf.set_string(x, area.y, "Category", header_style);
    }

    /// Render a single transaction row
    fn render_row(&self, area: Rect, buf: &mut Buffer, tx: &Transaction, selected: bool) {
        let date_w = 10;
        let amount_w = 12;
        let type_w = 8;
        let cat_w = 12;
        let desc_w = (area.width as usize).saturating_sub(date_w + amount_w + type_w + cat_w + 4);

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

        let mut x = area.x;

        // Date
        let date_str = tx.date.format("%Y-%m-%d").to_string();
        buf.set_string(x, area.y, &date_str, base_style);
        x += date_w as u16;

        // Description (truncate if needed)
        let desc = if tx.description.len() > desc_w {
            format!("{}...", &tx.description[..desc_w.saturating_sub(3)])
        } else {
            tx.description.clone()
        };
        buf.set_string(x, area.y, &desc, base_style);
        x += desc_w as u16;

        // Amount
        let amount_str = self.format_amount(tx.amount, &tx.transaction_type);
        let amount_style = if selected {
            self.type_style(&tx.transaction_type)
                .bg(self.theme.colors.surface)
        } else {
            self.type_style(&tx.transaction_type)
        };
        buf.set_string(x, area.y, &amount_str, amount_style);
        x += amount_w as u16;

        // Type
        let type_str = match tx.transaction_type {
            TransactionType::Income => "INC",
            TransactionType::Expense => "EXP",
            TransactionType::Transfer => "TRF",
        };
        buf.set_string(x, area.y, type_str, amount_style);
        x += type_w as u16;

        // Category (truncate if needed)
        let cat = if tx.category.len() > cat_w {
            format!("{}...", &tx.category[..cat_w.saturating_sub(3)])
        } else {
            tx.category.clone()
        };
        buf.set_string(x, area.y, &cat, base_style);
    }

    /// Render the summary footer
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let summary = self.state.month_summary.as_ref();
        let total_income = summary.map(|s| s.total_income).unwrap_or(Decimal::ZERO);
        let total_expenses = summary.map(|s| s.total_expenses).unwrap_or(Decimal::ZERO);
        let net = total_income - total_expenses;

        let net_color = if net >= Decimal::ZERO {
            self.theme.colors.success
        } else {
            self.theme.colors.error
        };

        let footer = Paragraph::new(Line::from(vec![
            Span::styled("Total: ", Style::default().fg(self.theme.colors.text_muted)),
            Span::styled(
                format!("{}${:.2}", if net >= Decimal::ZERO { "+" } else { "" }, net),
                Style::default().fg(net_color),
            ),
            Span::raw(" | "),
            Span::styled(
                "Income: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", total_income),
                Style::default().fg(self.theme.colors.success),
            ),
            Span::raw(" | "),
            Span::styled(
                "Expenses: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", total_expenses),
                Style::default().fg(self.theme.colors.error),
            ),
        ]))
        .alignment(Alignment::Center);
        footer.render(area, buf);
    }

    /// Render the add/edit form popup
    fn render_form(&self, area: Rect, buf: &mut Buffer) {
        let popup_width = 50;
        let popup_height = 18;
        let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        Clear.render(popup_area, buf);

        let title = if self.state.form.is_edit { " Edit Transaction " } else { " Add Transaction " };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.accent));
        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let active_style = Style::default().fg(self.theme.colors.accent);
        let normal_style = Style::default().fg(self.theme.colors.text_primary);

        // Date
        buf.set_string(inner.x + 2, inner.y + 1, "Date:", if self.state.form.active_field == 0 { active_style } else { normal_style });
        let date_rect = Rect::new(inner.x + 10, inner.y + 1, inner.width - 12, 1);
        let date_input = TextInput::new(&self.state.form.date, self.theme).placeholder("YYYY-MM-DD").block(Block::default());
        if self.state.form.active_field == 0 {
            let mut state = self.state.form.date.clone();
            state.focus();
            TextInput::new(&state, self.theme).placeholder("YYYY-MM-DD").block(Block::default()).render(date_rect, buf);
        } else {
            date_input.render(date_rect, buf);
        }

        // Description
        buf.set_string(inner.x + 2, inner.y + 4, "Desc:", if self.state.form.active_field == 1 { active_style } else { normal_style });
        let desc_rect = Rect::new(inner.x + 10, inner.y + 4, inner.width - 12, 1);
        let desc_input = TextInput::new(&self.state.form.description, self.theme).placeholder("Groceries").block(Block::default());
        if self.state.form.active_field == 1 {
            let mut state = self.state.form.description.clone();
            state.focus();
            TextInput::new(&state, self.theme).placeholder("Groceries").block(Block::default()).render(desc_rect, buf);
        } else {
            desc_input.render(desc_rect, buf);
        }

        // Amount
        buf.set_string(inner.x + 2, inner.y + 7, "Amount:", if self.state.form.active_field == 2 { active_style } else { normal_style });
        let amount_rect = Rect::new(inner.x + 10, inner.y + 7, inner.width - 12, 1);
        let amount_input = TextInput::new(&self.state.form.amount, self.theme).placeholder("50.00").block(Block::default());
        if self.state.form.active_field == 2 {
            let mut state = self.state.form.amount.clone();
            state.focus();
            TextInput::new(&state, self.theme).placeholder("50.00").block(Block::default()).render(amount_rect, buf);
        } else {
            amount_input.render(amount_rect, buf);
        }

        // Type
        buf.set_string(inner.x + 2, inner.y + 10, "Type:", if self.state.form.active_field == 3 { active_style } else { normal_style });
        let type_str = match transaction_types()[self.state.form.type_idx] {
            TransactionType::Income => "Income",
            TransactionType::Expense => "Expense",
            TransactionType::Transfer => "Transfer",
        };
        buf.set_string(inner.x + 10, inner.y + 10, format!("< {} >", type_str), if self.state.form.active_field == 3 { active_style } else { normal_style });

        // Category
        buf.set_string(inner.x + 2, inner.y + 12, "Category:", if self.state.form.active_field == 4 { active_style } else { normal_style });
        let cat_rect = Rect::new(inner.x + 12, inner.y + 12, inner.width - 14, 1);
        let cat_input = TextInput::new(&self.state.form.category, self.theme).placeholder("Food").block(Block::default());
        if self.state.form.active_field == 4 {
            let mut state = self.state.form.category.clone();
            state.focus();
            TextInput::new(&state, self.theme).placeholder("Food").block(Block::default()).render(cat_rect, buf);
        } else {
            cat_input.render(cat_rect, buf);
        }

        // Error message
        if let Some(err) = &self.state.form.error {
            buf.set_string(inner.x + 2, inner.y + 14, err, Style::default().fg(self.theme.colors.error));
        }

        // Footer
        buf.set_string(inner.x + 2, inner.y + 16, "Tab/Shift+Tab: move | Enter: save | Esc: cancel", Style::default().fg(self.theme.colors.text_muted));
    }
}

impl Widget for TransactionsView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Transactions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 5 {
            return;
        }

        // Layout: header | table header | table rows | footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Month header
                Constraint::Length(1), // Table header
                Constraint::Min(3),    // Table rows
                Constraint::Length(1), // Footer
            ])
            .split(inner);

        // Render header
        self.render_header(chunks[0], buf);

        // Render table header
        self.render_table_header(chunks[1], buf);

        // Get visible transactions
        let visible: Vec<(usize, &Transaction)> =
            if let Some(ref indices) = self.state.filtered_indices {
                indices
                    .iter()
                    .map(|&i| (i, &self.state.transactions[i]))
                    .collect()
            } else {
                self.state.transactions.iter().enumerate().collect()
            };

        if visible.is_empty() {
            let empty = Paragraph::new(if self.state.searching {
                "No matching transactions"
            } else {
                "No transactions this month"
            })
            .style(Style::default().fg(self.theme.colors.text_muted))
            .alignment(Alignment::Center);
            empty.render(chunks[2], buf);
        } else {
            // Render visible rows with scrolling
            let table_height = chunks[2].height as usize;
            let start = self.state.table_state.offset;
            let end = (start + table_height).min(visible.len());

            for (row_idx, (_, tx)) in visible.iter().enumerate().skip(start).take(end - start) {
                let y = chunks[2].y + (row_idx - start) as u16;
                let row_area = Rect::new(chunks[2].x, y, chunks[2].width, 1);
                let selected = row_idx == self.state.table_state.selected;
                self.render_row(row_area, buf, tx, selected);
            }
        }

        // Render footer
        self.render_footer(chunks[3], buf);

        if self.state.form.is_open {
            self.render_form(area, buf);
        }
    }
}
