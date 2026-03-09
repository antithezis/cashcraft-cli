//! Progress bar widget for budget utilization
//!
//! A progress bar widget that supports:
//! - Budget utilization display
//! - Color gradients (green -> yellow -> red)
//! - Animated filling
//! - Label display

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::ui::theme::Theme;

/// A progress bar widget for displaying budget utilization
pub struct ProgressBar<'a> {
    /// Progress value (0.0 to 1.0)
    value: f64,
    /// Optional label to display
    label: Option<&'a str>,
    /// Whether to show percentage
    show_percentage: bool,
    /// Theme for colors
    theme: &'a Theme,
    /// Custom color override
    color: Option<ratatui::style::Color>,
    /// Whether to invert colors (red=good, green=bad)
    inverted: bool,
    /// Progress bar style (default, thin, thick)
    style: ProgressStyle,
}

/// Progress bar visual style
#[derive(Debug, Clone, Copy, Default)]
pub enum ProgressStyle {
    /// Standard block characters
    #[default]
    Block,
    /// Thin line style
    Thin,
    /// ASCII-safe style
    Ascii,
    /// Gradient with smooth transitions
    Gradient,
}

impl<'a> ProgressBar<'a> {
    pub fn new(value: f64, theme: &'a Theme) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            label: None,
            show_percentage: true,
            theme,
            color: None,
            inverted: false,
            style: ProgressStyle::Block,
        }
    }

    /// Set the progress label
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set whether to show percentage
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set a custom color (overrides automatic color)
    pub fn color(mut self, color: ratatui::style::Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Invert colors (useful for savings where high = good)
    pub fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    /// Set the progress bar style
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the appropriate color based on value
    fn get_color(&self) -> ratatui::style::Color {
        if let Some(color) = self.color {
            return color;
        }

        let value = if self.inverted {
            1.0 - self.value
        } else {
            self.value
        };

        if value >= 0.9 {
            self.theme.colors.error
        } else if value > 0.75 {
            self.theme.colors.warning
        } else {
            self.theme.colors.success
        }
    }

    /// Get characters for the progress bar
    fn get_chars(&self) -> (&str, &str) {
        match self.style {
            ProgressStyle::Block => ("█", "░"),
            ProgressStyle::Thin => ("━", "─"),
            ProgressStyle::Ascii => ("#", "."),
            ProgressStyle::Gradient => ("█", "░"),
        }
    }
}

impl Widget for ProgressBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 1 {
            return;
        }

        let color = self.get_color();
        let (filled_char, empty_char) = self.get_chars();

        // Calculate space for label and percentage
        let label_width = self.label.map(|l| l.chars().count() + 1).unwrap_or(0) as u16;
        let pct_width = if self.show_percentage { 5 } else { 0 }; // " 100%"

        let bar_start = area.x + label_width;
        let bar_width = area.width.saturating_sub(label_width + pct_width);

        if bar_width < 3 {
            return;
        }

        // Render label
        if let Some(label) = self.label {
            let label_style = Style::default().fg(self.theme.colors.text_secondary);
            buf.set_string(area.x, area.y, label, label_style);
        }

        // Calculate filled portion
        let filled = ((bar_width as f64 * self.value) as u16).min(bar_width);

        // Render filled portion
        let filled_style = Style::default().fg(color);
        for x in 0..filled {
            buf.set_string(bar_start + x, area.y, filled_char, filled_style);
        }

        // Render empty portion
        let empty_style = Style::default().fg(self.theme.colors.text_muted);
        for x in filled..bar_width {
            buf.set_string(bar_start + x, area.y, empty_char, empty_style);
        }

        // Render percentage
        if self.show_percentage {
            let pct = format!("{:>3}%", (self.value * 100.0) as u32);
            let pct_x = bar_start + bar_width + 1;
            let pct_style = Style::default().fg(self.theme.colors.text_secondary);
            buf.set_string(pct_x, area.y, &pct, pct_style);
        }
    }
}

/// A mini progress bar for inline display (e.g., in tables)
pub struct MiniProgress<'a> {
    value: f64,
    width: u16,
    theme: &'a Theme,
}

impl<'a> MiniProgress<'a> {
    pub fn new(value: f64, width: u16, theme: &'a Theme) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            width,
            theme,
        }
    }

    fn get_color(&self) -> ratatui::style::Color {
        if self.value > 0.9 {
            self.theme.colors.error
        } else if self.value > 0.75 {
            self.theme.colors.warning
        } else {
            self.theme.colors.success
        }
    }
}

impl Widget for MiniProgress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = self.width.min(area.width);
        if width == 0 {
            return;
        }

        let filled = ((width as f64 * self.value) as u16).min(width);
        let color = self.get_color();

        let filled_style = Style::default().fg(color);
        let empty_style = Style::default().fg(self.theme.colors.text_muted);

        for x in 0..filled {
            buf.set_string(area.x + x, area.y, "▮", filled_style);
        }
        for x in filled..width {
            buf.set_string(area.x + x, area.y, "▯", empty_style);
        }
    }
}

/// Budget progress widget with amount display
pub struct BudgetProgress<'a> {
    /// Category name
    category: &'a str,
    /// Amount spent
    spent: f64,
    /// Budget limit
    budget: f64,
    /// Theme
    theme: &'a Theme,
    /// Currency symbol
    currency: &'a str,
}

impl<'a> BudgetProgress<'a> {
    pub fn new(category: &'a str, spent: f64, budget: f64, theme: &'a Theme) -> Self {
        Self {
            category,
            spent,
            budget,
            theme,
            currency: "$",
        }
    }

    pub fn currency(mut self, currency: &'a str) -> Self {
        self.currency = currency;
        self
    }
}

impl Widget for BudgetProgress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 1 {
            return;
        }

        let value = if self.budget > 0.0 {
            (self.spent / self.budget).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Color based on budget utilization
        let color = if value > 0.9 {
            self.theme.colors.error
        } else if value > 0.75 {
            self.theme.colors.warning
        } else {
            self.theme.colors.success
        };

        // Layout: [Category] [========--] $spent/$budget (XX%)
        let category_width = 12;
        let amount_width = 20;
        let bar_width = area.width.saturating_sub(category_width + amount_width + 2);

        // Category name
        let cat_text: String = if self.category.chars().count() > category_width as usize {
            format!(
                "{}…",
                self.category
                    .chars()
                    .take(category_width as usize - 1)
                    .collect::<String>()
            )
        } else {
            format!("{:<width$}", self.category, width = category_width as usize)
        };
        buf.set_string(
            area.x,
            area.y,
            &cat_text,
            Style::default().fg(self.theme.colors.text_primary),
        );

        // Progress bar
        let bar_start = area.x + category_width + 1;
        let filled = ((bar_width as f64 * value) as u16).min(bar_width);

        for x in 0..filled {
            buf.set_string(bar_start + x, area.y, "█", Style::default().fg(color));
        }
        for x in filled..bar_width {
            buf.set_string(
                bar_start + x,
                area.y,
                "░",
                Style::default().fg(self.theme.colors.text_muted),
            );
        }

        // Amount text
        let amount_text = format!(
            "{}{:.0}/{}{:.0} ({:.0}%)",
            self.currency,
            self.spent,
            self.currency,
            self.budget,
            value * 100.0
        );
        let amount_x = bar_start + bar_width + 1;
        buf.set_string(
            amount_x,
            area.y,
            &amount_text,
            Style::default().fg(self.theme.colors.text_secondary),
        );
    }
}

/// Circular/ring progress indicator
pub struct CircularProgress<'a> {
    value: f64,
    label: Option<&'a str>,
    theme: &'a Theme,
}

impl<'a> CircularProgress<'a> {
    pub fn new(value: f64, theme: &'a Theme) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            label: None,
            theme,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    fn get_color(&self) -> ratatui::style::Color {
        if self.value > 0.9 {
            self.theme.colors.error
        } else if self.value > 0.75 {
            self.theme.colors.warning
        } else {
            self.theme.colors.success
        }
    }
}

impl Widget for CircularProgress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Circular progress needs at least 5x3 area
        if area.width < 5 || area.height < 3 {
            return;
        }

        let color = self.get_color();

        // ASCII representation of a circular progress
        // Using characters to simulate a circle
        let segments = 8;
        let filled_segments = ((segments as f64 * self.value) as usize).min(segments);

        // Simple representation: [○○○●●●●●] or similar
        let chars: Vec<&str> = (0..segments)
            .map(|i| if i < filled_segments { "●" } else { "○" })
            .collect();

        let progress_str = chars.join("");
        let center_x = area.x + (area.width.saturating_sub(progress_str.len() as u16)) / 2;

        buf.set_string(center_x, area.y, &progress_str, Style::default().fg(color));

        // Percentage below
        let pct = format!("{:.0}%", self.value * 100.0);
        let pct_x = area.x + (area.width.saturating_sub(pct.len() as u16)) / 2;
        buf.set_string(
            pct_x,
            area.y + 1,
            &pct,
            Style::default().fg(self.theme.colors.text_secondary),
        );

        // Label below percentage
        if let Some(label) = self.label {
            if area.height > 2 {
                let label_width = label.chars().count() as u16;
                let label_x = area.x + (area.width.saturating_sub(label_width)) / 2;
                buf.set_string(
                    label_x,
                    area.y + 2,
                    label,
                    Style::default().fg(self.theme.colors.text_muted),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    fn test_theme() -> Theme {
        Theme::dracula()
    }

    #[test]
    fn test_progress_bar_value_clamping() {
        let theme = test_theme();

        let bar1 = ProgressBar::new(-0.5, &theme);
        assert_eq!(bar1.value, 0.0);

        let bar2 = ProgressBar::new(1.5, &theme);
        assert_eq!(bar2.value, 1.0);

        let bar3 = ProgressBar::new(0.5, &theme);
        assert_eq!(bar3.value, 0.5);
    }

    #[test]
    fn test_progress_bar_colors() {
        let theme = test_theme();

        // Low value = success
        let bar_low = ProgressBar::new(0.3, &theme);
        assert_eq!(bar_low.get_color(), theme.colors.success);

        // Medium value = warning
        let bar_med = ProgressBar::new(0.8, &theme);
        assert_eq!(bar_med.get_color(), theme.colors.warning);

        // High value = error
        let bar_high = ProgressBar::new(0.95, &theme);
        assert_eq!(bar_high.get_color(), theme.colors.error);
    }

    #[test]
    fn test_progress_bar_inverted() {
        let theme = test_theme();

        // Inverted: low value = error
        let bar = ProgressBar::new(0.1, &theme).inverted(true);
        assert_eq!(bar.get_color(), theme.colors.error);
    }

    #[test]
    fn test_progress_bar_render() {
        let theme = test_theme();
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);

        let bar = ProgressBar::new(0.5, &theme);
        bar.render(area, &mut buf);

        // Should have rendered something
        // (We can't easily check exact output without comparing buffer contents)
    }

    #[test]
    fn test_budget_progress() {
        let theme = test_theme();
        let area = Rect::new(0, 0, 50, 1);
        let mut buf = Buffer::empty(area);

        let progress = BudgetProgress::new("Food", 300.0, 500.0, &theme);
        progress.render(area, &mut buf);

        // Should have rendered without panic
    }
}
