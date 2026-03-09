//! Chart widget for financial data visualization
//!
//! ASCII/Unicode charts for:
//! - Bar charts for expenses by category
//! - Line charts for trends over time
//! - Sparklines for compact trend display
//! - Pie charts for distribution

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::ui::theme::Theme;

/// Data point for charts
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

impl DataPoint {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

/// A bar chart widget
pub struct BarChart<'a> {
    data: Vec<DataPoint>,
    theme: &'a Theme,
    title: Option<&'a str>,
    max_value: Option<f64>,
    show_values: bool,
    horizontal: bool,
    bar_width: u16,
}

impl<'a> BarChart<'a> {
    pub fn new(data: Vec<DataPoint>, theme: &'a Theme) -> Self {
        Self {
            data,
            theme,
            title: None,
            max_value: None,
            show_values: true,
            horizontal: false,
            bar_width: 3,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn max_value(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    pub fn bar_width(mut self, width: u16) -> Self {
        self.bar_width = width;
        self
    }

    fn get_max(&self) -> f64 {
        self.max_value.unwrap_or_else(|| {
            self.data
                .iter()
                .map(|d| d.value)
                .fold(0.0, f64::max)
                .max(1.0)
        })
    }
}

impl Widget for BarChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 5 || self.data.is_empty() {
            return;
        }

        if self.horizontal {
            self.render_horizontal(area, buf);
        } else {
            self.render_vertical(area, buf);
        }
    }
}

impl<'a> BarChart<'a> {
    fn render_vertical(self, area: Rect, buf: &mut Buffer) {
        let max = self.get_max();
        let title_height = if self.title.is_some() { 1 } else { 0 };
        let label_height = 1;
        let value_height = if self.show_values { 1 } else { 0 };
        let chart_height = area
            .height
            .saturating_sub(title_height + label_height + value_height);

        // Render title
        if let Some(title) = self.title {
            let title_style = Style::default().fg(self.theme.colors.text_primary);
            buf.set_string(area.x, area.y, title, title_style);
        }

        let chart_y = area.y + title_height;
        let label_y = chart_y + chart_height;
        let value_y = label_y + 1;

        // Calculate bar positions
        let total_bar_width = self.bar_width + 1; // bar + gap
        let visible_bars = (area.width / total_bar_width) as usize;
        let bars_to_show = self.data.len().min(visible_bars);

        // Color palette for bars
        let colors = [
            self.theme.colors.primary,
            self.theme.colors.secondary,
            self.theme.colors.accent,
            self.theme.colors.info,
            self.theme.colors.success,
            self.theme.colors.warning,
        ];

        for (i, point) in self.data.iter().take(bars_to_show).enumerate() {
            let bar_x = area.x + (i as u16 * total_bar_width);
            let bar_color = colors[i % colors.len()];

            // Calculate bar height
            let ratio = point.value / max;
            let bar_height = ((chart_height as f64 * ratio) as u16).min(chart_height);

            // Draw bar
            let bar_style = Style::default().fg(bar_color);
            for y in 0..bar_height {
                let draw_y = chart_y + chart_height - 1 - y;
                for x in 0..self.bar_width {
                    buf.set_string(bar_x + x, draw_y, "█", bar_style);
                }
            }

            // Draw label (truncated)
            let label: String = point.label.chars().take(self.bar_width as usize).collect();
            let label_style = Style::default().fg(self.theme.colors.text_muted);
            buf.set_string(bar_x, label_y, &label, label_style);

            // Draw value
            if self.show_values && value_y < area.y + area.height {
                let value_text = format_value(point.value);
                let value_truncated: String =
                    value_text.chars().take(self.bar_width as usize).collect();
                let value_style = Style::default().fg(self.theme.colors.text_secondary);
                buf.set_string(bar_x, value_y, &value_truncated, value_style);
            }
        }
    }

    fn render_horizontal(self, area: Rect, buf: &mut Buffer) {
        let max = self.get_max();
        let title_height = if self.title.is_some() { 1 } else { 0 };

        // Render title
        if let Some(title) = self.title {
            let title_style = Style::default().fg(self.theme.colors.text_primary);
            buf.set_string(area.x, area.y, title, title_style);
        }

        let chart_y = area.y + title_height;
        let label_width = 10;
        let value_width = if self.show_values { 8 } else { 0 };
        let bar_area_width = area.width.saturating_sub(label_width + value_width + 2);

        let visible_bars = (area.height - title_height) as usize;
        let bars_to_show = self.data.len().min(visible_bars);

        let colors = [
            self.theme.colors.primary,
            self.theme.colors.secondary,
            self.theme.colors.accent,
            self.theme.colors.info,
        ];

        for (i, point) in self.data.iter().take(bars_to_show).enumerate() {
            let row_y = chart_y + i as u16;
            let bar_color = colors[i % colors.len()];

            // Draw label
            let label: String = if point.label.chars().count() > label_width as usize {
                format!(
                    "{}…",
                    point
                        .label
                        .chars()
                        .take(label_width as usize - 1)
                        .collect::<String>()
                )
            } else {
                format!("{:<width$}", point.label, width = label_width as usize)
            };
            let label_style = Style::default().fg(self.theme.colors.text_secondary);
            buf.set_string(area.x, row_y, &label, label_style);

            // Calculate bar width
            let ratio = point.value / max;
            let bar_width = ((bar_area_width as f64 * ratio) as u16).min(bar_area_width);

            // Draw bar
            let bar_x = area.x + label_width + 1;
            let bar_style = Style::default().fg(bar_color);
            for x in 0..bar_width {
                buf.set_string(bar_x + x, row_y, "█", bar_style);
            }

            // Draw value
            if self.show_values {
                let value_text = format_value(point.value);
                let value_x = bar_x + bar_area_width + 1;
                let value_style = Style::default().fg(self.theme.colors.text_muted);
                buf.set_string(value_x, row_y, &value_text, value_style);
            }
        }
    }
}

/// A line chart widget
pub struct LineChart<'a> {
    data: Vec<f64>,
    labels: Vec<String>,
    theme: &'a Theme,
    title: Option<&'a str>,
    show_dots: bool,
    fill: bool,
}

impl<'a> LineChart<'a> {
    pub fn new(data: Vec<f64>, theme: &'a Theme) -> Self {
        Self {
            data,
            labels: Vec::new(),
            theme,
            title: None,
            show_dots: true,
            fill: false,
        }
    }

    pub fn labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn show_dots(mut self, show: bool) -> Self {
        self.show_dots = show;
        self
    }

    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }
}

impl Widget for LineChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 5 || self.data.is_empty() {
            return;
        }

        let title_height = if self.title.is_some() { 1 } else { 0 };
        let label_height = 1;
        let chart_height = area.height.saturating_sub(title_height + label_height) as usize;
        let chart_width = area.width as usize;

        // Render title
        if let Some(title) = self.title {
            let title_style = Style::default().fg(self.theme.colors.text_primary);
            buf.set_string(area.x, area.y, title, title_style);
        }

        let chart_y = area.y + title_height;
        let label_y = chart_y + chart_height as u16;

        // Find min/max for scaling
        let max = self.data.iter().copied().fold(f64::MIN, f64::max);
        let min = self.data.iter().copied().fold(f64::MAX, f64::min);
        let range = (max - min).max(1.0);

        // Calculate x positions
        let x_step = chart_width as f64 / (self.data.len().max(2) - 1) as f64;

        // Draw the line using braille-like characters
        let line_style = Style::default().fg(self.theme.colors.primary);
        let fill_style = Style::default().fg(self.theme.colors.surface);

        let mut prev_y: Option<u16> = None;

        for (i, &value) in self.data.iter().enumerate() {
            let x = area.x + (i as f64 * x_step) as u16;
            if x >= area.x + area.width {
                break;
            }

            // Scale value to chart height
            let normalized = (value - min) / range;
            let y_offset = (chart_height as f64 * (1.0 - normalized)) as u16;
            let y = chart_y + y_offset.min(chart_height as u16 - 1);

            // Draw dot
            if self.show_dots {
                buf.set_string(x, y, "●", line_style);
            }

            // Draw line segment
            if let Some(prev) = prev_y {
                // Connect with previous point
                let dy = y as i32 - prev as i32;
                let steps = dy.unsigned_abs().max(1);
                let y_step = dy.signum();

                let prev_x = if i > 0 {
                    area.x + ((i - 1) as f64 * x_step) as u16
                } else {
                    x
                };

                let x_diff = x.saturating_sub(prev_x);
                for step in 0..steps {
                    let intermediate_y = prev as i32 + (step as i32 + 1) * y_step;
                    let intermediate_x = prev_x + (x_diff * step as u16 / steps as u16);

                    if intermediate_y >= chart_y as i32
                        && intermediate_y < (chart_y + chart_height as u16) as i32
                    {
                        let char = if y_step > 0 {
                            "╲"
                        } else if y_step < 0 {
                            "╱"
                        } else {
                            "─"
                        };
                        buf.set_string(intermediate_x, intermediate_y as u16, char, line_style);
                    }
                }
            }

            // Fill below line
            if self.fill {
                for fill_y in (y + 1)..(chart_y + chart_height as u16) {
                    buf.set_string(x, fill_y, "░", fill_style);
                }
            }

            prev_y = Some(y);
        }

        // Draw labels
        if !self.labels.is_empty() {
            let label_style = Style::default().fg(self.theme.colors.text_muted);
            for (i, label) in self.labels.iter().enumerate() {
                let x = area.x + (i as f64 * x_step) as u16;
                if x >= area.x + area.width.saturating_sub(3) {
                    break;
                }
                let truncated: String = label.chars().take(4).collect();
                buf.set_string(x, label_y, &truncated, label_style);
            }
        }
    }
}

/// A sparkline for compact inline charts
pub struct Sparkline<'a> {
    data: Vec<f64>,
    theme: &'a Theme,
    show_bounds: bool,
}

impl<'a> Sparkline<'a> {
    pub fn new(data: Vec<f64>, theme: &'a Theme) -> Self {
        Self {
            data,
            theme,
            show_bounds: false,
        }
    }

    pub fn show_bounds(mut self, show: bool) -> Self {
        self.show_bounds = show;
        self
    }
}

impl Widget for Sparkline<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || self.data.is_empty() {
            return;
        }

        // Sparkline characters (8 levels)
        let blocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        let max = self.data.iter().copied().fold(f64::MIN, f64::max);
        let min = self.data.iter().copied().fold(f64::MAX, f64::min);
        let range = (max - min).max(0.001);

        let bounds_width = if self.show_bounds { 6 } else { 0 };
        let chart_width = area.width.saturating_sub(bounds_width) as usize;

        // Determine trend color
        let trend_up = self.data.last().unwrap_or(&0.0) >= self.data.first().unwrap_or(&0.0);
        let color = if trend_up {
            self.theme.colors.success
        } else {
            self.theme.colors.error
        };

        let style = Style::default().fg(color);

        // Sample data to fit width
        let step = self.data.len() as f64 / chart_width as f64;

        for i in 0..chart_width {
            let idx = (i as f64 * step) as usize;
            if idx >= self.data.len() {
                break;
            }

            let value = self.data[idx];
            let normalized = (value - min) / range;
            let block_idx = (normalized * 7.0) as usize;
            let block = blocks[block_idx.min(7)];

            buf.set_string(area.x + i as u16, area.y, block.to_string(), style);
        }

        // Show bounds
        if self.show_bounds {
            let bounds_x = area.x + chart_width as u16 + 1;
            let bounds_style = Style::default().fg(self.theme.colors.text_muted);

            let max_str = format_compact(max);
            let min_str = format_compact(min);

            buf.set_string(bounds_x, area.y, &max_str, bounds_style);
            if area.height > 1 {
                buf.set_string(bounds_x, area.y + 1, &min_str, bounds_style);
            }
        }
    }
}

/// A simple pie/donut chart using ASCII art
pub struct PieChart<'a> {
    segments: Vec<DataPoint>,
    theme: &'a Theme,
    title: Option<&'a str>,
    show_legend: bool,
}

impl<'a> PieChart<'a> {
    pub fn new(segments: Vec<DataPoint>, theme: &'a Theme) -> Self {
        Self {
            segments,
            theme,
            title: None,
            show_legend: true,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }
}

impl Widget for PieChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 8 || self.segments.is_empty() {
            return;
        }

        let title_height = if self.title.is_some() { 1 } else { 0 };

        // Render title
        if let Some(title) = self.title {
            let title_style = Style::default().fg(self.theme.colors.text_primary);
            buf.set_string(area.x, area.y, title, title_style);
        }

        let chart_y = area.y + title_height;
        let total: f64 = self.segments.iter().map(|s| s.value).sum();

        if total <= 0.0 {
            return;
        }

        // Color palette
        let colors = [
            self.theme.colors.primary,
            self.theme.colors.secondary,
            self.theme.colors.accent,
            self.theme.colors.info,
            self.theme.colors.success,
            self.theme.colors.warning,
        ];

        // Simple ASCII pie representation using segments
        // Note: segment_chars reserved for future enhanced pie visualization
        let _segment_chars = ['█', '▓', '▒', '░', '▄', '▀', '▌', '▐'];
        let pie_width = if self.show_legend {
            area.width / 2
        } else {
            area.width
        };

        // Draw a horizontal bar representation (simpler than actual pie)
        let bar_y = chart_y + 2;
        let bar_width = pie_width.saturating_sub(4);

        let mut x_offset = 0u16;
        for (i, segment) in self.segments.iter().enumerate() {
            let ratio = segment.value / total;
            let seg_width = ((bar_width as f64 * ratio) as u16).max(1);
            let color = colors[i % colors.len()];
            let style = Style::default().fg(color);

            for x in 0..seg_width {
                if x_offset + x < bar_width {
                    buf.set_string(area.x + 2 + x_offset + x, bar_y, "█", style);
                }
            }
            x_offset += seg_width;
        }

        // Draw legend
        if self.show_legend {
            let legend_x = area.x + pie_width + 2;
            let legend_style = Style::default().fg(self.theme.colors.text_secondary);

            for (i, segment) in self.segments.iter().enumerate() {
                let legend_y = chart_y + i as u16;
                if legend_y >= area.y + area.height {
                    break;
                }

                let color = colors[i % colors.len()];
                let pct = segment.value / total * 100.0;

                // Color indicator
                buf.set_string(legend_x, legend_y, "■", Style::default().fg(color));

                // Label and percentage
                let label = format!(" {} ({:.0}%)", truncate_str(&segment.label, 12), pct);
                buf.set_string(legend_x + 1, legend_y, &label, legend_style);
            }
        }
    }
}

/// Combined income/expense comparison chart
pub struct FinanceChart<'a> {
    income: Vec<DataPoint>,
    expenses: Vec<DataPoint>,
    theme: &'a Theme,
    title: Option<&'a str>,
}

impl<'a> FinanceChart<'a> {
    pub fn new(income: Vec<DataPoint>, expenses: Vec<DataPoint>, theme: &'a Theme) -> Self {
        Self {
            income,
            expenses,
            theme,
            title: None,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

impl Widget for FinanceChart<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 8 {
            return;
        }

        let title_height = if self.title.is_some() { 1 } else { 0 };
        let legend_height = 1;

        // Render title
        if let Some(title) = self.title {
            let title_style = Style::default().fg(self.theme.colors.text_primary);
            buf.set_string(area.x, area.y, title, title_style);
        }

        let chart_y = area.y + title_height;
        let chart_height = area.height.saturating_sub(title_height + legend_height + 1);
        let chart_width = area.width;

        // Find max value across both series
        let max = self
            .income
            .iter()
            .chain(self.expenses.iter())
            .map(|d| d.value)
            .fold(0.0, f64::max)
            .max(1.0);

        let n_points = self.income.len().max(self.expenses.len());
        if n_points == 0 {
            return;
        }

        let x_step = chart_width as f64 / n_points as f64;

        let income_style = Style::default().fg(self.theme.colors.chart_income);
        let expense_style = Style::default().fg(self.theme.colors.chart_expense);

        // Draw both series
        for i in 0..n_points {
            let x = area.x + (i as f64 * x_step) as u16;
            if x >= area.x + chart_width {
                break;
            }

            // Income bar
            if i < self.income.len() {
                let ratio = self.income[i].value / max;
                let bar_height = ((chart_height as f64 * ratio) as u16).min(chart_height);
                for y in 0..bar_height {
                    let draw_y = chart_y + chart_height - 1 - y;
                    buf.set_string(x, draw_y, "▌", income_style);
                }
            }

            // Expense bar (offset by 1)
            if i < self.expenses.len() && x + 1 < area.x + chart_width {
                let ratio = self.expenses[i].value / max;
                let bar_height = ((chart_height as f64 * ratio) as u16).min(chart_height);
                for y in 0..bar_height {
                    let draw_y = chart_y + chart_height - 1 - y;
                    buf.set_string(x + 1, draw_y, "▐", expense_style);
                }
            }
        }

        // Legend
        let legend_y = chart_y + chart_height + 1;
        if legend_y < area.y + area.height {
            buf.set_string(area.x, legend_y, "▌", income_style);
            buf.set_string(
                area.x + 1,
                legend_y,
                " Income",
                Style::default().fg(self.theme.colors.text_muted),
            );

            buf.set_string(area.x + 12, legend_y, "▐", expense_style);
            buf.set_string(
                area.x + 13,
                legend_y,
                " Expenses",
                Style::default().fg(self.theme.colors.text_muted),
            );
        }
    }
}

/// Format a value for display
fn format_value(value: f64) -> String {
    if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}K", value / 1_000.0)
    } else if value.fract() == 0.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}

/// Format for compact display
fn format_compact(value: f64) -> String {
    if value >= 1_000.0 {
        format!("{:.0}K", value / 1_000.0)
    } else {
        format!("{:.0}", value)
    }
}

/// Truncate a string to max length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max_len - 1).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::dracula()
    }

    #[test]
    fn test_data_point() {
        let point = DataPoint::new("Test", 100.0);
        assert_eq!(point.label, "Test");
        assert_eq!(point.value, 100.0);
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(500.0), "500");
        assert_eq!(format_value(1500.0), "1.5K");
        assert_eq!(format_value(1500000.0), "1.5M");
        assert_eq!(format_value(99.99), "99.99");
    }

    #[test]
    fn test_bar_chart_render() {
        let theme = test_theme();
        let data = vec![
            DataPoint::new("A", 100.0),
            DataPoint::new("B", 200.0),
            DataPoint::new("C", 150.0),
        ];
        let area = Rect::new(0, 0, 40, 20);
        let mut buf = Buffer::empty(area);

        let chart = BarChart::new(data, &theme).title("Test");
        chart.render(area, &mut buf);

        // Should render without panic
    }

    #[test]
    fn test_sparkline_render() {
        let theme = test_theme();
        let data = vec![1.0, 2.0, 3.0, 2.5, 4.0, 3.5, 5.0];
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);

        let sparkline = Sparkline::new(data, &theme);
        sparkline.render(area, &mut buf);

        // Should render without panic
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("Hello", 10), "Hello");
        assert_eq!(truncate_str("Hello World", 8), "Hello W…");
    }
}
