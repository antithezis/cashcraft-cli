//! Income view
//!
//! Displays and manages income sources with:
//! - Income sources list
//! - Add/Edit/Delete operations
//! - Variable name display ($name)
//! - Frequency and amount display

use crate::ui::widgets::{InputState, TextInput};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use rust_decimal::Decimal;

use crate::domain::income::{Frequency, IncomeSource};
use crate::repository::Database;
use crate::services::IncomeService;
use crate::ui::theme::Theme;
use crate::ui::widgets::TableState;

pub fn frequencies() -> &'static [Frequency] {
    &[
        Frequency::Daily,
        Frequency::Weekly,
        Frequency::BiWeekly,
        Frequency::Monthly,
        Frequency::Quarterly,
        Frequency::Yearly,
        Frequency::OneTime,
    ]
}

/// Form state for adding/editing income
#[derive(Debug, Clone)]
pub struct IncomeFormState {
    pub is_open: bool,
    pub is_edit: bool,
    pub active_field: usize, // 0: var_name, 1: display_name, 2: amount, 3: frequency
    pub var_name: InputState,
    pub display_name: InputState,
    pub amount: InputState,
    pub frequency_idx: usize,
    pub error: Option<String>,
    pub edit_id: Option<String>,
}

impl Default for IncomeFormState {
    fn default() -> Self {
        Self {
            is_open: false,
            is_edit: false,
            active_field: 0,
            var_name: InputState::new(),
            display_name: InputState::new(),
            amount: InputState::new(),
            frequency_idx: 3, // Monthly
            error: None,
            edit_id: None,
        }
    }
}

/// Income view state
#[derive(Debug, Clone)]
pub struct IncomeState {
    /// Table state for navigation
    pub table_state: TableState,
    /// Cached income sources
    pub income_sources: Vec<IncomeSource>,
    /// Total monthly income
    pub total_monthly: Decimal,
    /// Form state
    pub form: IncomeFormState,
}

impl Default for IncomeState {
    fn default() -> Self {
        Self::new()
    }
}

impl IncomeState {
    /// Create a new income view state
    pub fn new() -> Self {
        Self {
            table_state: TableState::new(),
            income_sources: Vec::new(),
            total_monthly: Decimal::ZERO,
            form: IncomeFormState::default(),
        }
    }

    /// Refresh income data from database
    pub fn refresh(&mut self, db: &Database) {
        let service = IncomeService::new(db);
        self.income_sources = service.get_all().unwrap_or_default();
        self.total_monthly = service.total_monthly_income().unwrap_or(Decimal::ZERO);
        self.table_state.set_total(self.income_sources.len());
    }

    /// Get currently selected income source
    pub fn selected(&self) -> Option<&IncomeSource> {
        self.income_sources.get(self.table_state.selected)
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

/// Income view widget
pub struct IncomeView<'a> {
    state: &'a IncomeState,
    theme: &'a Theme,
}

impl<'a> IncomeView<'a> {
    pub fn new(state: &'a IncomeState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Render the header summary
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Income Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(inner);

        // Total Monthly Income
        let total = Paragraph::new(Line::from(vec![
            Span::styled(
                "Total Monthly: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", self.state.total_monthly),
                Style::default()
                    .fg(self.theme.colors.success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center);
        total.render(chunks[0], buf);

        // Active sources count
        let active_count = self
            .state
            .income_sources
            .iter()
            .filter(|i| i.is_active)
            .count();
        let sources = Paragraph::new(Line::from(vec![
            Span::styled(
                "Active: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("{}/{}", active_count, self.state.income_sources.len()),
                Style::default().fg(self.theme.colors.text_primary),
            ),
        ]))
        .alignment(Alignment::Center);
        sources.render(chunks[1], buf);

        // Yearly projection
        let yearly = self.state.total_monthly * Decimal::from(12);
        let yearly_text = Paragraph::new(Line::from(vec![
            Span::styled(
                "Yearly: ",
                Style::default().fg(self.theme.colors.text_muted),
            ),
            Span::styled(
                format!("${:.2}", yearly),
                Style::default().fg(self.theme.colors.text_primary),
            ),
        ]))
        .alignment(Alignment::Center);
        yearly_text.render(chunks[2], buf);
    }

    /// Render the income sources list
    fn render_list(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Income Sources ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.state.income_sources.is_empty() {
            let empty = Paragraph::new("No income sources. Press [a] to add one.")
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
                Constraint::Length(15), // Amount
                Constraint::Length(12), // Freq
                Constraint::Length(15), // Monthly
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

            let headers = ["", "Variable", "Name", "Amount", "Freq", "Monthly", "Act"];
            let alignments = [
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
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

        // Render income rows
        let rows_area = Rect {
            y: inner.y + 1,
            height: inner.height.saturating_sub(1),
            ..inner
        };

        let visible_rows = rows_area.height as usize;
        let start = self.state.table_state.offset;

        for (i, income) in self
            .state
            .income_sources
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
            } else if income.is_active {
                Style::default().fg(self.theme.colors.text_primary)
            } else {
                Style::default().fg(self.theme.colors.text_muted)
            };

            // Background for selection
            if is_selected {
                buf.set_style(row_area, style);
            }

            // 0. Cursor
            Paragraph::new(if is_selected { " ▶" } else { "" })
                .style(style)
                .render(cols[0], buf);

            // 1. Variable
            Paragraph::new(format!("${}", income.variable_name))
                .style(style)
                .alignment(Alignment::Left)
                .render(cols[1], buf);

            // 2. Name
            Paragraph::new(income.display_name.as_str())
                .style(style)
                .alignment(Alignment::Left)
                .render(cols[2], buf);

            // 3. Amount
            Paragraph::new(format!("${:.2}", income.amount))
                .style(style)
                .alignment(Alignment::Right)
                .render(cols[3], buf);

            // 4. Frequency
            Paragraph::new(format_frequency(&income.frequency))
                .style(style)
                .alignment(Alignment::Left)
                .render(cols[4], buf);

            // 5. Monthly
            Paragraph::new(format!("${:.2}", income.monthly_amount()))
                .style(style)
                .alignment(Alignment::Right)
                .render(cols[5], buf);

            // 6. Active
            Paragraph::new(if income.is_active { "●" } else { "○" })
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
            Span::styled("[j/k]", Style::default().fg(self.theme.colors.primary)),
            Span::raw(" nav "),
            Span::styled("[?]", Style::default().fg(self.theme.colors.primary)),
            Span::raw(" help"),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(self.theme.colors.text_muted));

        help.render(area, buf);
    }

    /// Render the add/edit form popup
    fn render_form(&self, area: Rect, buf: &mut Buffer) {
        let popup_width = 50;
        let popup_height = 16;
        let x = (area.width.saturating_sub(popup_width)) / 2 + area.x;
        let y = (area.height.saturating_sub(popup_height)) / 2 + area.y;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        Clear.render(popup_area, buf);

        let title = if self.state.form.is_edit {
            " Edit Income "
        } else {
            " Add Income "
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
            .placeholder("salary")
            .block(Block::default());
        if self.state.form.active_field == 0 {
            let mut state = self.state.form.var_name.clone();
            state.focus();
            TextInput::new(&state, self.theme)
                .placeholder("salary")
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
            .placeholder("Primary Job")
            .block(Block::default());
        if self.state.form.active_field == 1 {
            let mut state = self.state.form.display_name.clone();
            state.focus();
            TextInput::new(&state, self.theme)
                .placeholder("Primary Job")
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

        // Frequency (Selector)
        buf.set_string(
            inner.x + 2,
            inner.y + 10,
            "Freq:",
            if self.state.form.active_field == 3 {
                active_style
            } else {
                normal_style
            },
        );
        let freq_str = format_frequency(&frequencies()[self.state.form.frequency_idx]);
        buf.set_string(
            inner.x + 12,
            inner.y + 10,
            format!("< {} >", freq_str),
            if self.state.form.active_field == 3 {
                active_style
            } else {
                normal_style
            },
        );

        // Error message
        if let Some(err) = &self.state.form.error {
            buf.set_string(
                inner.x + 2,
                inner.y + 12,
                err,
                Style::default().fg(self.theme.colors.error),
            );
        }

        // Footer
        buf.set_string(
            inner.x + 2,
            inner.y + 14,
            "Tab/Shift+Tab: move | Enter: save | Esc: cancel",
            Style::default().fg(self.theme.colors.text_muted),
        );
    }
}

impl Widget for IncomeView<'_> {
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

/// Format frequency for display
pub fn format_frequency(freq: &Frequency) -> String {
    match freq {
        Frequency::Daily => "Daily".to_string(),
        Frequency::Weekly => "Weekly".to_string(),
        Frequency::BiWeekly => "Bi-Weekly".to_string(),
        Frequency::Monthly => "Monthly".to_string(),
        Frequency::Quarterly => "Quarterly".to_string(),
        Frequency::Yearly => "Yearly".to_string(),
        Frequency::OneTime => "One-Time".to_string(),
    }
}
