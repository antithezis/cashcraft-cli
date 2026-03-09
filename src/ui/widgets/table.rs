//! Table widget with Vim navigation
//!
//! A custom table widget that supports:
//! - Vim-style j/k navigation
//! - Column sorting
//! - Selection highlighting
//! - Scrolling with viewport
//! - Search/filter mode

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use crate::ui::theme::Theme;

/// State for the VimTable widget
#[derive(Debug, Clone)]
pub struct TableState {
    /// Currently selected row index
    pub selected: usize,
    /// Scroll offset (first visible row)
    pub offset: usize,
    /// Total number of rows
    total: usize,
    /// Visible height in rows
    visible_height: usize,
}

impl Default for TableState {
    fn default() -> Self {
        Self::new()
    }
}

impl TableState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            offset: 0,
            total: 0,
            visible_height: 0,
        }
    }

    /// Set the total number of rows
    pub fn set_total(&mut self, total: usize) {
        self.total = total;
        if self.selected >= total && total > 0 {
            self.selected = total - 1;
        }
    }

    /// Set the visible height
    pub fn set_visible_height(&mut self, height: usize) {
        self.visible_height = height;
        self.ensure_visible();
    }

    /// Select a specific index
    pub fn select(&mut self, index: usize) {
        if self.total > 0 {
            self.selected = index.min(self.total.saturating_sub(1));
            self.ensure_visible();
        }
    }

    /// Move selection down (j)
    pub fn next(&mut self) {
        if self.total > 0 {
            self.selected = (self.selected + 1).min(self.total - 1);
            self.ensure_visible();
        }
    }

    /// Move selection up (k)
    pub fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.ensure_visible();
    }

    /// Jump to first row (gg)
    pub fn first(&mut self) {
        self.selected = 0;
        self.offset = 0;
    }

    /// Jump to last row (G)
    pub fn last(&mut self) {
        if self.total > 0 {
            self.selected = self.total - 1;
            self.ensure_visible();
        }
    }

    /// Move half page down (Ctrl+d)
    pub fn half_page_down(&mut self) {
        let half = self.visible_height / 2;
        if self.total > 0 {
            self.selected = (self.selected + half).min(self.total - 1);
            self.ensure_visible();
        }
    }

    /// Move half page up (Ctrl+u)
    pub fn half_page_up(&mut self) {
        let half = self.visible_height / 2;
        self.selected = self.selected.saturating_sub(half);
        self.ensure_visible();
    }

    /// Move full page down (Ctrl+f)
    pub fn page_down(&mut self) {
        if self.total > 0 {
            self.selected = (self.selected + self.visible_height).min(self.total - 1);
            self.ensure_visible();
        }
    }

    /// Move full page up (Ctrl+b)
    pub fn page_up(&mut self) {
        self.selected = self.selected.saturating_sub(self.visible_height);
        self.ensure_visible();
    }

    /// Ensure selected row is visible in viewport
    fn ensure_visible(&mut self) {
        if self.visible_height == 0 {
            return;
        }

        // If selected is above viewport, scroll up
        if self.selected < self.offset {
            self.offset = self.selected;
        }

        // If selected is below viewport, scroll down
        if self.selected >= self.offset + self.visible_height {
            self.offset = self.selected.saturating_sub(self.visible_height - 1);
        }
    }

    /// Get the selected index
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get the scroll offset
    pub fn offset(&self) -> usize {
        self.offset
    }
}

/// A row in the table
#[derive(Debug, Clone)]
pub struct TableRow {
    cells: Vec<String>,
    style: Option<Style>,
}

impl TableRow {
    pub fn new(cells: Vec<String>) -> Self {
        Self { cells, style: None }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
}

impl From<Vec<String>> for TableRow {
    fn from(cells: Vec<String>) -> Self {
        Self::new(cells)
    }
}

impl From<Vec<&str>> for TableRow {
    fn from(cells: Vec<&str>) -> Self {
        Self::new(cells.into_iter().map(String::from).collect())
    }
}

/// Column definition for the table
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub header: String,
    pub width: ColumnWidth,
    pub alignment: Alignment,
}

/// Column width specification
#[derive(Debug, Clone, Copy)]
pub enum ColumnWidth {
    /// Fixed width in characters
    Fixed(u16),
    /// Percentage of available space
    Percent(u16),
    /// Fill remaining space equally
    Fill,
}

/// Text alignment within a column
#[derive(Debug, Clone, Copy, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

impl TableColumn {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            width: ColumnWidth::Fill,
            alignment: Alignment::Left,
        }
    }

    pub fn fixed_width(mut self, width: u16) -> Self {
        self.width = ColumnWidth::Fixed(width);
        self
    }

    pub fn percent_width(mut self, percent: u16) -> Self {
        self.width = ColumnWidth::Percent(percent);
        self
    }

    pub fn fill(mut self) -> Self {
        self.width = ColumnWidth::Fill;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn right(mut self) -> Self {
        self.alignment = Alignment::Right;
        self
    }

    pub fn center(mut self) -> Self {
        self.alignment = Alignment::Center;
        self
    }
}

/// A table widget with Vim-style navigation
pub struct VimTable<'a> {
    rows: Vec<TableRow>,
    columns: Vec<TableColumn>,
    theme: &'a Theme,
    title: Option<String>,
    block: Option<Block<'a>>,
    highlight_symbol: &'a str,
    show_header: bool,
}

impl<'a> VimTable<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            theme,
            title: None,
            block: None,
            highlight_symbol: "▶ ",
            show_header: true,
        }
    }

    pub fn rows(mut self, rows: Vec<TableRow>) -> Self {
        self.rows = rows;
        self
    }

    pub fn columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns = columns;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = symbol;
        self
    }

    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Calculate column widths based on available space
    fn calculate_widths(&self, available_width: u16) -> Vec<u16> {
        if self.columns.is_empty() {
            return vec![];
        }

        let highlight_width = self.highlight_symbol.chars().count() as u16;
        let available = available_width.saturating_sub(highlight_width);

        let mut widths = vec![0u16; self.columns.len()];
        let mut remaining = available;
        let mut fill_count = 0;

        // First pass: allocate fixed and percentage widths
        for (i, col) in self.columns.iter().enumerate() {
            match col.width {
                ColumnWidth::Fixed(w) => {
                    widths[i] = w.min(remaining);
                    remaining = remaining.saturating_sub(widths[i]);
                }
                ColumnWidth::Percent(p) => {
                    widths[i] = (available as u32 * p.min(100) as u32 / 100) as u16;
                    remaining = remaining.saturating_sub(widths[i]);
                }
                ColumnWidth::Fill => {
                    fill_count += 1;
                }
            }
        }

        // Second pass: distribute remaining space to fill columns
        if fill_count > 0 {
            let fill_width = remaining / fill_count;
            for (i, col) in self.columns.iter().enumerate() {
                if matches!(col.width, ColumnWidth::Fill) {
                    widths[i] = fill_width;
                }
            }
        }

        widths
    }

    /// Align text within a given width
    fn align_text(&self, text: &str, width: usize, alignment: Alignment) -> String {
        let text_len = text.chars().count();
        if text_len >= width {
            return text
                .chars()
                .take(width.saturating_sub(1))
                .collect::<String>()
                + "…";
        }

        match alignment {
            Alignment::Left => format!("{:<width$}", text),
            Alignment::Center => format!("{:^width$}", text),
            Alignment::Right => format!("{:>width$}", text),
        }
    }
}

impl<'a> StatefulWidget for VimTable<'a> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Get inner area if block is set
        let inner = if let Some(ref block) = self.block {
            let inner = block.inner(area);
            block.clone().render(area, buf);
            inner
        } else {
            area
        };

        if inner.width < 3 || inner.height < 1 {
            return;
        }

        // Update state with current dimensions
        let header_height = if self.show_header { 1 } else { 0 };
        let content_height = inner.height.saturating_sub(header_height) as usize;
        state.set_total(self.rows.len());
        state.set_visible_height(content_height);

        let widths = self.calculate_widths(inner.width);
        let highlight_width = self.highlight_symbol.chars().count() as u16;

        // Render header
        let mut y = inner.y;
        if self.show_header && !self.columns.is_empty() {
            let header_style = Style::default()
                .fg(self.theme.colors.text_secondary)
                .add_modifier(Modifier::BOLD);

            // Empty space for highlight symbol
            buf.set_string(
                inner.x,
                y,
                " ".repeat(highlight_width as usize),
                Style::default(),
            );

            let mut x = inner.x + highlight_width;
            for (i, col) in self.columns.iter().enumerate() {
                if x >= inner.x + inner.width {
                    break;
                }
                let text = self.align_text(&col.header, widths[i] as usize, Alignment::Left);
                buf.set_string(x, y, &text, header_style);
                x += widths[i];
            }
            y += 1;
        }

        // Render rows
        for (i, row_idx) in (state.offset..).take(content_height).enumerate() {
            if row_idx >= self.rows.len() {
                break;
            }

            let row = &self.rows[row_idx];
            let is_selected = row_idx == state.selected;
            let row_y = y + i as u16;

            // Row style
            let base_style = if is_selected {
                Style::default()
                    .fg(self.theme.colors.text_primary)
                    .bg(self.theme.colors.surface)
                    .add_modifier(Modifier::BOLD)
            } else {
                row.style
                    .unwrap_or_else(|| Style::default().fg(self.theme.colors.text_primary))
            };

            // Highlight symbol
            let symbol = if is_selected {
                self.highlight_symbol
            } else {
                &" ".repeat(highlight_width as usize)
            };
            let symbol_style = if is_selected {
                Style::default()
                    .fg(self.theme.colors.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            buf.set_string(inner.x, row_y, symbol, symbol_style);

            // Render cells
            let mut x = inner.x + highlight_width;
            for (col_idx, cell) in row.cells.iter().enumerate() {
                if col_idx >= widths.len() || x >= inner.x + inner.width {
                    break;
                }

                let alignment = self
                    .columns
                    .get(col_idx)
                    .map(|c| c.alignment)
                    .unwrap_or_default();
                let text = self.align_text(cell, widths[col_idx] as usize, alignment);
                buf.set_string(x, row_y, &text, base_style);
                x += widths[col_idx];
            }

            // Fill rest of row if selected (for highlighting)
            if is_selected {
                while x < inner.x + inner.width {
                    buf.set_string(x, row_y, " ", base_style);
                    x += 1;
                }
            }
        }
    }
}

/// Helper to create a simple table widget without explicit column definitions
pub struct SimpleTable<'a> {
    rows: Vec<Vec<String>>,
    headers: Vec<String>,
    theme: &'a Theme,
    title: Option<String>,
}

impl<'a> SimpleTable<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            rows: Vec::new(),
            headers: Vec::new(),
            theme,
            title: None,
        }
    }

    pub fn headers(mut self, headers: Vec<impl Into<String>>) -> Self {
        self.headers = headers.into_iter().map(Into::into).collect();
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn build(self) -> VimTable<'a> {
        let columns: Vec<TableColumn> = self
            .headers
            .into_iter()
            .map(|h| TableColumn::new(h))
            .collect();

        let rows: Vec<TableRow> = self.rows.into_iter().map(TableRow::new).collect();

        let mut table = VimTable::new(self.theme).columns(columns).rows(rows);

        if let Some(title) = self.title {
            table = table.block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(self.theme.colors.border)),
            );
        }

        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_state_navigation() {
        let mut state = TableState::new();
        state.set_total(10);
        state.set_visible_height(5);

        assert_eq!(state.selected(), 0);

        state.next();
        assert_eq!(state.selected(), 1);

        state.previous();
        assert_eq!(state.selected(), 0);

        state.previous(); // Should stay at 0
        assert_eq!(state.selected(), 0);

        state.last();
        assert_eq!(state.selected(), 9);

        state.first();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn test_table_state_page_navigation() {
        let mut state = TableState::new();
        state.set_total(100);
        state.set_visible_height(10);

        state.half_page_down();
        assert_eq!(state.selected(), 5);

        state.page_down();
        assert_eq!(state.selected(), 15);

        state.page_up();
        assert_eq!(state.selected(), 5);

        state.half_page_up();
        assert_eq!(state.selected(), 0);
    }

    #[test]
    fn test_table_state_scroll() {
        let mut state = TableState::new();
        state.set_total(20);
        state.set_visible_height(5);

        // Move past visible area
        state.select(10);
        assert!(state.offset() <= 10);
        assert!(state.offset() + 5 > 10);

        // Jump to end
        state.last();
        assert_eq!(state.selected(), 19);
        assert!(state.offset() + 5 > 19);
    }

    #[test]
    fn test_column_width() {
        let col = TableColumn::new("Test").fixed_width(10).right();

        assert!(matches!(col.width, ColumnWidth::Fixed(10)));
        assert!(matches!(col.alignment, Alignment::Right));
    }

    #[test]
    fn test_table_row_from() {
        let row: TableRow = vec!["a", "b", "c"].into();
        assert_eq!(row.cells.len(), 3);
        assert_eq!(row.cells[0], "a");
    }
}
