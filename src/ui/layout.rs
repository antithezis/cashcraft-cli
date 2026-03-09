//! Layout management for CashCraft TUI
//!
//! Provides layout helpers for consistent spacing, panel arrangement,
//! and responsive constraints across all views.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Layout constants for consistent spacing
pub const PADDING: u16 = 1;
pub const BORDER_WIDTH: u16 = 1;
pub const MIN_CONTENT_WIDTH: u16 = 40;
pub const MIN_CONTENT_HEIGHT: u16 = 10;

/// Create main layout with header, content, and footer
///
/// # Layout Structure
/// ```text
/// +----------------------------------+
/// |           Header (3)             |
/// +----------------------------------+
/// |                                  |
/// |         Content (flex)           |
/// |                                  |
/// +----------------------------------+
/// |           Footer (3)             |
/// +----------------------------------+
/// ```
pub fn main_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with title and tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer with status and help
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}

/// Create two-panel horizontal layout (list + detail)
///
/// # Arguments
/// * `area` - The area to split
/// * `left_percent` - Percentage of width for left panel (0-100)
///
/// # Layout Structure
/// ```text
/// +------------+---------------------+
/// |            |                     |
/// |   Left     |       Right         |
/// |   (%)      |       (%)           |
/// |            |                     |
/// +------------+---------------------+
/// ```
pub fn split_horizontal(area: Rect, left_percent: u16) -> (Rect, Rect) {
    let left_pct = left_percent.min(100);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(100 - left_pct),
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create two-panel vertical layout
///
/// # Arguments
/// * `area` - The area to split
/// * `top_percent` - Percentage of height for top panel (0-100)
///
/// # Layout Structure
/// ```text
/// +----------------------------------+
/// |              Top (%)             |
/// +----------------------------------+
/// |            Bottom (%)            |
/// +----------------------------------+
/// ```
pub fn split_vertical(area: Rect, top_percent: u16) -> (Rect, Rect) {
    let top_pct = top_percent.min(100);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(top_pct),
            Constraint::Percentage(100 - top_pct),
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create three-panel horizontal layout
///
/// # Layout Structure
/// ```text
/// +--------+--------------+--------+
/// |  Left  |    Center    | Right  |
/// |  (%)   |     (%)      |  (%)   |
/// +--------+--------------+--------+
/// ```
pub fn split_three_horizontal(
    area: Rect,
    left_percent: u16,
    center_percent: u16,
) -> (Rect, Rect, Rect) {
    let left_pct = left_percent.min(100);
    let center_pct = center_percent.min(100 - left_pct);
    let right_pct = 100 - left_pct - center_pct;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left_pct),
            Constraint::Percentage(center_pct),
            Constraint::Percentage(right_pct),
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}

/// Center a fixed-size rectangle within an area
///
/// If the requested size is larger than the area, the area dimensions are used.
///
/// # Arguments
/// * `area` - The containing area
/// * `width` - Desired width of centered rect
/// * `height` - Desired height of centered rect
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let actual_width = width.min(area.width);
    let actual_height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(actual_width)) / 2;
    let y = area.y + (area.height.saturating_sub(actual_height)) / 2;
    Rect::new(x, y, actual_width, actual_height)
}

/// Center horizontally with specified height percentage
pub fn centered_horizontal(area: Rect, width: u16) -> Rect {
    let actual_width = width.min(area.width);
    let x = area.x + (area.width.saturating_sub(actual_width)) / 2;
    Rect::new(x, area.y, actual_width, area.height)
}

/// Center vertically with specified width percentage
pub fn centered_vertical(area: Rect, height: u16) -> Rect {
    let actual_height = height.min(area.height);
    let y = area.y + (area.height.saturating_sub(actual_height)) / 2;
    Rect::new(area.x, y, area.width, actual_height)
}

/// Create area with uniform margin/padding
///
/// # Arguments
/// * `area` - The original area
/// * `margin` - Margin size on all sides
pub fn with_margin(area: Rect, margin: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(margin),
        area.y.saturating_add(margin),
        area.width.saturating_sub(margin * 2),
        area.height.saturating_sub(margin * 2),
    )
}

/// Create area with asymmetric margins
///
/// # Arguments
/// * `area` - The original area
/// * `horizontal` - Horizontal margin (left and right)
/// * `vertical` - Vertical margin (top and bottom)
pub fn with_margin_asymmetric(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(horizontal),
        area.y.saturating_add(vertical),
        area.width.saturating_sub(horizontal * 2),
        area.height.saturating_sub(vertical * 2),
    )
}

/// Create area with individual margins
///
/// # Arguments
/// * `area` - The original area
/// * `top` - Top margin
/// * `right` - Right margin
/// * `bottom` - Bottom margin
/// * `left` - Left margin
pub fn with_margin_individual(area: Rect, top: u16, right: u16, bottom: u16, left: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(left),
        area.y.saturating_add(top),
        area.width.saturating_sub(left + right),
        area.height.saturating_sub(top + bottom),
    )
}

/// Create a modal/dialog area centered in the given area
///
/// # Arguments
/// * `area` - The containing area
/// * `width_percent` - Width as percentage of container
/// * `height_percent` - Height as percentage of container
pub fn modal(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let width = (area.width as u32 * width_percent.min(100) as u32 / 100) as u16;
    let height = (area.height as u32 * height_percent.min(100) as u32 / 100) as u16;
    centered(area, width, height)
}

/// Create a fixed-height row layout
///
/// Returns a vector of Rects, each with the specified height.
/// Useful for creating lists or form layouts.
pub fn rows(area: Rect, row_height: u16, count: usize) -> Vec<Rect> {
    let mut rows = Vec::with_capacity(count);
    let mut y = area.y;

    for _ in 0..count {
        if y + row_height > area.y + area.height {
            break;
        }
        rows.push(Rect::new(area.x, y, area.width, row_height));
        y += row_height;
    }

    rows
}

/// Create a fixed-width column layout
///
/// Returns a vector of Rects, each with the specified width.
/// Useful for creating card layouts or grid-like displays.
pub fn columns(area: Rect, col_width: u16, count: usize) -> Vec<Rect> {
    let mut cols = Vec::with_capacity(count);
    let mut x = area.x;

    for _ in 0..count {
        if x + col_width > area.x + area.width {
            break;
        }
        cols.push(Rect::new(x, area.y, col_width, area.height));
        x += col_width;
    }

    cols
}

/// Create a sidebar + main content layout
///
/// # Layout Structure
/// ```text
/// +--------+------------------------+
/// |        |                        |
/// | Side   |       Main             |
/// | bar    |      Content           |
/// | (20%)  |       (80%)            |
/// |        |                        |
/// +--------+------------------------+
/// ```
pub fn sidebar_layout(area: Rect, sidebar_width: u16) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(sidebar_width.min(100)),
            Constraint::Percentage((100_u16).saturating_sub(sidebar_width)),
        ])
        .split(area);

    (chunks[0], chunks[1])
}

/// Create a form layout with labels and inputs
///
/// # Arguments
/// * `area` - The form container area
/// * `label_width` - Fixed width for labels
/// * `field_count` - Number of form fields
///
/// Returns pairs of (label_rect, input_rect) for each field
pub fn form_layout(area: Rect, label_width: u16, field_count: usize) -> Vec<(Rect, Rect)> {
    let row_height = 3; // Height for input with borders
    let mut fields = Vec::with_capacity(field_count);
    let mut y = area.y;

    for _ in 0..field_count {
        if y + row_height > area.y + area.height {
            break;
        }

        let row = Rect::new(area.x, y, area.width, row_height);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(label_width), Constraint::Min(0)])
            .split(row);

        fields.push((chunks[0], chunks[1]));
        y += row_height;
    }

    fields
}

/// Calculate the visible range for a scrollable list
///
/// # Arguments
/// * `total_items` - Total number of items in the list
/// * `visible_height` - Height of the visible area in rows
/// * `selected` - Currently selected index
///
/// # Returns
/// * (start_index, end_index) - The range of items to render
pub fn scroll_range(total_items: usize, visible_height: usize, selected: usize) -> (usize, usize) {
    if total_items == 0 {
        return (0, 0);
    }

    let half_visible = visible_height / 2;

    // Try to center the selected item
    let start = if selected <= half_visible {
        0
    } else if selected >= total_items.saturating_sub(half_visible) {
        total_items.saturating_sub(visible_height)
    } else {
        selected.saturating_sub(half_visible)
    };

    let end = (start + visible_height).min(total_items);

    (start, end)
}

/// Check if an area has minimum usable dimensions
pub fn is_usable(area: Rect) -> bool {
    area.width >= MIN_CONTENT_WIDTH && area.height >= MIN_CONTENT_HEIGHT
}

/// Constraint builder for common layouts
pub struct ConstraintBuilder {
    constraints: Vec<Constraint>,
}

impl ConstraintBuilder {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    pub fn fixed(mut self, length: u16) -> Self {
        self.constraints.push(Constraint::Length(length));
        self
    }

    pub fn min(mut self, length: u16) -> Self {
        self.constraints.push(Constraint::Min(length));
        self
    }

    pub fn max(mut self, length: u16) -> Self {
        self.constraints.push(Constraint::Max(length));
        self
    }

    pub fn percentage(mut self, pct: u16) -> Self {
        self.constraints.push(Constraint::Percentage(pct));
        self
    }

    pub fn ratio(mut self, num: u32, den: u32) -> Self {
        self.constraints.push(Constraint::Ratio(num, den));
        self
    }

    pub fn flex(mut self) -> Self {
        self.constraints.push(Constraint::Min(0));
        self
    }

    pub fn build(self) -> Vec<Constraint> {
        self.constraints
    }

    /// Apply constraints horizontally to an area
    pub fn split_horizontal(self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(self.constraints)
            .split(area)
            .to_vec()
    }

    /// Apply constraints vertically to an area
    pub fn split_vertical(self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(self.constraints)
            .split(area)
            .to_vec()
    }
}

impl Default for ConstraintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_area() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn test_main_layout() {
        let area = test_area();
        let (header, content, footer) = main_layout(area);

        assert_eq!(header.height, 3);
        assert_eq!(footer.height, 3);
        assert_eq!(content.height, 44); // 50 - 3 - 3
        assert_eq!(header.y, 0);
        assert_eq!(content.y, 3);
        assert_eq!(footer.y, 47);
    }

    #[test]
    fn test_split_horizontal() {
        let area = test_area();
        let (left, right) = split_horizontal(area, 30);

        assert_eq!(left.width, 30);
        assert_eq!(right.width, 70);
        assert_eq!(left.x, 0);
        assert_eq!(right.x, 30);
    }

    #[test]
    fn test_split_vertical() {
        let area = test_area();
        let (top, bottom) = split_vertical(area, 40);

        assert_eq!(top.height, 20); // 40% of 50
        assert_eq!(bottom.height, 30);
        assert_eq!(top.y, 0);
        assert_eq!(bottom.y, 20);
    }

    #[test]
    fn test_centered() {
        let area = test_area();
        let c = centered(area, 40, 20);

        assert_eq!(c.width, 40);
        assert_eq!(c.height, 20);
        assert_eq!(c.x, 30); // (100 - 40) / 2
        assert_eq!(c.y, 15); // (50 - 20) / 2
    }

    #[test]
    fn test_centered_larger_than_area() {
        let area = test_area();
        let c = centered(area, 200, 100);

        // Should be clamped to area dimensions
        assert_eq!(c.width, 100);
        assert_eq!(c.height, 50);
        assert_eq!(c.x, 0);
        assert_eq!(c.y, 0);
    }

    #[test]
    fn test_with_margin() {
        let area = test_area();
        let m = with_margin(area, 5);

        assert_eq!(m.x, 5);
        assert_eq!(m.y, 5);
        assert_eq!(m.width, 90);
        assert_eq!(m.height, 40);
    }

    #[test]
    fn test_modal() {
        let area = test_area();
        let m = modal(area, 50, 50);

        assert_eq!(m.width, 50);
        assert_eq!(m.height, 25);
        assert_eq!(m.x, 25);
        assert_eq!(m.y, 12);
    }

    #[test]
    fn test_scroll_range() {
        // Small list, no scroll needed
        let (start, end) = scroll_range(5, 10, 2);
        assert_eq!((start, end), (0, 5));

        // Large list, scroll to middle
        let (start, end) = scroll_range(100, 10, 50);
        assert_eq!(start, 45);
        assert_eq!(end, 55);

        // At beginning
        let (start, end) = scroll_range(100, 10, 0);
        assert_eq!(start, 0);
        assert_eq!(end, 10);

        // At end
        let (start, end) = scroll_range(100, 10, 99);
        assert_eq!(start, 90);
        assert_eq!(end, 100);
    }

    #[test]
    fn test_constraint_builder() {
        let constraints = ConstraintBuilder::new().fixed(3).flex().fixed(3).build();

        assert_eq!(constraints.len(), 3);
    }

    #[test]
    fn test_rows() {
        let area = test_area();
        let rows = rows(area, 5, 20);

        // Should fit 10 rows (50 / 5)
        assert_eq!(rows.len(), 10);
        assert_eq!(rows[0].height, 5);
        assert_eq!(rows[0].y, 0);
        assert_eq!(rows[1].y, 5);
    }

    #[test]
    fn test_is_usable() {
        assert!(is_usable(Rect::new(0, 0, 100, 50)));
        assert!(!is_usable(Rect::new(0, 0, 30, 50))); // Too narrow
        assert!(!is_usable(Rect::new(0, 0, 100, 5))); // Too short
    }
}
