//! Settings view
//!
//! Application configuration including:
//! - Theme selection
//! - Currency format
//! - Date format
//! - Data export/import options

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::config::Settings;
use crate::ui::theme::Theme;
use crate::ui::widgets::TableState;

/// Settings section
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SettingsSection {
    #[default]
    Appearance,
    Format,
    Data,
    About,
}

impl SettingsSection {
    /// Get all sections
    pub fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::Appearance,
            SettingsSection::Format,
            SettingsSection::Data,
            SettingsSection::About,
        ]
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Format => "Format",
            SettingsSection::Data => "Data",
            SettingsSection::About => "About",
        }
    }
}

/// State for the settings view
#[derive(Debug, Clone)]
pub struct SettingsState {
    /// Current section
    pub section: SettingsSection,
    /// Table state for navigation
    pub table_state: TableState,
    /// Current settings
    pub settings: Settings,
    /// Available theme names
    pub theme_names: Vec<String>,
    /// Selected theme index
    pub selected_theme: usize,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsState {
    /// Create new settings state
    pub fn new() -> Self {
        let theme_names: Vec<String> = Theme::available_themes()
            .iter()
            .map(|t| t.to_string())
            .collect();

        Self {
            section: SettingsSection::Appearance,
            table_state: TableState::new(),
            settings: Settings::default(),
            selected_theme: 0,
            theme_names,
        }
    }

    /// Load settings
    pub fn load(&mut self, settings: &Settings) {
        self.settings = settings.clone();

        // Find current theme index (settings.appearance.theme)
        self.selected_theme = self
            .theme_names
            .iter()
            .position(|n| n == &settings.appearance.theme)
            .unwrap_or(0);
    }

    /// Next section
    pub fn next_section(&mut self) {
        self.section = match self.section {
            SettingsSection::Appearance => SettingsSection::Format,
            SettingsSection::Format => SettingsSection::Data,
            SettingsSection::Data => SettingsSection::About,
            SettingsSection::About => SettingsSection::Appearance,
        };
    }

    /// Previous section
    pub fn prev_section(&mut self) {
        self.section = match self.section {
            SettingsSection::Appearance => SettingsSection::About,
            SettingsSection::Format => SettingsSection::Appearance,
            SettingsSection::Data => SettingsSection::Format,
            SettingsSection::About => SettingsSection::Data,
        };
    }

    /// Select next theme
    pub fn next_theme(&mut self) {
        if !self.theme_names.is_empty() {
            self.selected_theme = (self.selected_theme + 1) % self.theme_names.len();
            self.settings.appearance.theme = self.theme_names[self.selected_theme].clone();
        }
    }

    /// Select previous theme
    pub fn prev_theme(&mut self) {
        if !self.theme_names.is_empty() {
            self.selected_theme = if self.selected_theme == 0 {
                self.theme_names.len() - 1
            } else {
                self.selected_theme - 1
            };
            self.settings.appearance.theme = self.theme_names[self.selected_theme].clone();
        }
    }

    /// Navigation - in settings, next goes to next item in current section
    pub fn next(&mut self) {
        self.table_state.next();
    }
    pub fn previous(&mut self) {
        self.table_state.previous();
    }
}

/// Settings view widget
pub struct SettingsView<'a> {
    state: &'a SettingsState,
    theme: &'a Theme,
}

impl<'a> SettingsView<'a> {
    /// Create new settings view
    pub fn new(state: &'a SettingsState, theme: &'a Theme) -> Self {
        Self { state, theme }
    }

    /// Render section tabs
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let mut spans = Vec::new();

        for (i, section) in SettingsSection::all().iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }

            let style = if *section == self.state.section {
                Style::default()
                    .fg(self.theme.colors.accent)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(self.theme.colors.text_muted)
            };

            spans.push(Span::styled(section.name(), style));
        }

        let tabs = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        tabs.render(area, buf);
    }

    /// Render appearance section
    fn render_appearance(&self, area: Rect, buf: &mut Buffer) {
        let settings = [
            (
                "Theme",
                self.state.settings.appearance.theme.clone(),
                "h/l to change",
            ),
            (
                "Animations",
                if self.state.settings.appearance.animations_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
                .to_string(),
                "Enter to toggle",
            ),
        ];

        for (i, (label, value, hint)) in settings.iter().enumerate() {
            let y = area.y + i as u16 * 2;
            if y >= area.y + area.height {
                break;
            }

            let selected = i == self.state.table_state.selected;
            let style = if selected {
                Style::default()
                    .bg(self.theme.colors.surface)
                    .fg(self.theme.colors.text_primary)
            } else {
                Style::default().fg(self.theme.colors.text_primary)
            };

            // Label
            buf.set_string(area.x, y, format!("{}:", label), style);

            // Value
            buf.set_string(
                area.x + 20,
                y,
                value,
                Style::default()
                    .fg(self.theme.colors.accent)
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );

            // Hint
            if selected {
                buf.set_string(
                    area.x,
                    y + 1,
                    format!("  ({})", hint),
                    Style::default().fg(self.theme.colors.text_muted),
                );
            }
        }
    }

    /// Render format section
    fn render_format(&self, area: Rect, buf: &mut Buffer) {
        let settings = [
            (
                "Currency Symbol",
                self.state.settings.general.currency_symbol.clone(),
            ),
            (
                "Date Format",
                self.state.settings.general.date_format.clone(),
            ),
            (
                "Decimal Places",
                self.state.settings.playground.decimal_places.to_string(),
            ),
        ];

        for (i, (label, value)) in settings.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            buf.set_string(
                area.x,
                y,
                format!("{}: ", label),
                Style::default().fg(self.theme.colors.text_primary),
            );

            buf.set_string(
                area.x + 20,
                y,
                value,
                Style::default().fg(self.theme.colors.accent),
            );
        }
    }

    /// Render data section
    fn render_data(&self, area: Rect, buf: &mut Buffer) {
        let options = [
            ("Export Data", "Export all data to JSON"),
            ("Import Data", "Import data from JSON"),
            ("Export CSV", "Export transactions to CSV"),
            ("Backup Database", "Create database backup"),
        ];

        for (i, (label, desc)) in options.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            let selected = i == self.state.table_state.selected;
            let style = if selected {
                Style::default()
                    .bg(self.theme.colors.surface)
                    .fg(self.theme.colors.text_primary)
            } else {
                Style::default().fg(self.theme.colors.text_primary)
            };

            buf.set_string(area.x, y, format!("[{}]", label), style);
            buf.set_string(
                area.x + 20,
                y,
                *desc,
                Style::default().fg(self.theme.colors.text_muted),
            );
        }
    }

    /// Render about section
    fn render_about(&self, area: Rect, buf: &mut Buffer) {
        let about_text = vec![
            Line::from(vec![
                Span::styled(
                    "CashCraft",
                    Style::default()
                        .fg(self.theme.colors.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" v1.0.0"),
            ]),
            Line::from(""),
            Line::from("A Vim-powered TUI personal finance manager"),
            Line::from("Built with Rust and Ratatui"),
            Line::from(""),
            Line::from(Span::styled(
                "Keybindings:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  hjkl    - Navigate"),
            Line::from("  gh      - Go to Dashboard"),
            Line::from("  gt      - Go to Transactions"),
            Line::from("  gi      - Go to Income"),
            Line::from("  ge      - Go to Expenses"),
            Line::from("  gb      - Go to Budget"),
            Line::from("  gp      - Go to Playground"),
            Line::from("  gg      - Go to Charts"),
            Line::from("  gs      - Go to Settings"),
            Line::from("  :q      - Quit"),
        ];

        let about = Paragraph::new(about_text).alignment(Alignment::Left);
        about.render(area, buf);
    }
}

impl Widget for SettingsView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Settings ")
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
                Constraint::Length(2), // Tabs
                Constraint::Min(3),    // Content
            ])
            .split(inner);

        // Render tabs
        self.render_tabs(chunks[0], buf);

        // Render section content
        let content_area = Rect::new(
            chunks[1].x + 2,
            chunks[1].y + 1,
            chunks[1].width.saturating_sub(4),
            chunks[1].height.saturating_sub(2),
        );

        match self.state.section {
            SettingsSection::Appearance => self.render_appearance(content_area, buf),
            SettingsSection::Format => self.render_format(content_area, buf),
            SettingsSection::Data => self.render_data(content_area, buf),
            SettingsSection::About => self.render_about(content_area, buf),
        }
    }
}
