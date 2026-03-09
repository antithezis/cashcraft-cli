//! Application state and core logic
//!
//! The `App` struct is the central state container for CashCraft, managing:
//! - Current mode (Normal, Insert, Command)
//! - Current view (Dashboard, Transactions, etc.)
//! - Settings and theme
//! - Command buffer for command mode
//! - Status messages

use crate::config::{Keybindings, Settings};
use crate::ui::history::History;
use crate::ui::theme::Theme;

/// Pending action awaiting confirmation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    DeleteIncome(String),
    DeleteExpense(String),
    DeleteTransaction(String),
    DeleteBudget(String),
}

/// Vim-style editing mode
///
/// CashCraft uses a modal interface inspired by Vim:
/// - **Normal**: Navigation and commands (default)
/// - **Insert**: Data entry and text editing
/// - **Command**: Execute colon commands (`:q`, `:w`, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Navigation and command mode (default)
    ///
    /// Use hjkl for navigation, shortcuts for actions.
    /// Press `i` to enter Insert mode, `:` for Command mode.
    #[default]
    Normal,

    /// Data entry and text editing mode
    ///
    /// Type text directly into forms and inputs.
    /// Press `Esc` to return to Normal mode.
    Insert,

    /// Command execution mode
    ///
    /// Enter commands like `:q`, `:w`, `:export csv`.
    /// Press `Enter` to execute, `Esc` to cancel.
    Command,
}

impl Mode {
    /// Get the status bar indicator for this mode
    pub fn indicator(&self) -> &'static str {
        match self {
            Mode::Normal => "[N]",
            Mode::Insert => "[I]",
            Mode::Command => "[:]",
        }
    }

    /// Get the full name of this mode
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
    }
}

/// Application view/screen
///
/// Each view represents a distinct screen or feature area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Main dashboard with overview
    #[default]
    Dashboard,

    /// Income sources management
    Income,

    /// Expenses management
    Expenses,

    /// Transaction history and entry
    Transactions,

    /// Budget planning and tracking
    Budget,

    /// Playground calculator
    Playground,

    /// Charts and analytics
    Charts,

    /// Application settings
    Settings,
}

impl View {
    /// Get the shortcut hint for navigating to this view
    pub fn shortcut(&self) -> &'static str {
        match self {
            View::Dashboard => "gh",
            View::Income => "gi",
            View::Expenses => "ge",
            View::Transactions => "gt",
            View::Budget => "gb",
            View::Playground => "gp",
            View::Charts => "gg",
            View::Settings => "gs",
        }
    }

    /// Get the display name of this view
    pub fn name(&self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Income => "Income",
            View::Expenses => "Expenses",
            View::Transactions => "Transactions",
            View::Budget => "Budget",
            View::Playground => "Playground",
            View::Charts => "Charts",
            View::Settings => "Settings",
        }
    }

    /// Get all available views
    pub fn all() -> &'static [View] {
        &[
            View::Dashboard,
            View::Income,
            View::Expenses,
            View::Transactions,
            View::Budget,
            View::Playground,
            View::Charts,
            View::Settings,
        ]
    }
}

/// Status message severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSeverity {
    Info,
    Success,
    Warning,
    Error,
}

/// Status message with severity
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub severity: StatusSeverity,
}

impl StatusMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            severity: StatusSeverity::Info,
        }
    }

    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            severity: StatusSeverity::Success,
        }
    }

    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            severity: StatusSeverity::Warning,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            severity: StatusSeverity::Error,
        }
    }
}

/// Central application state
///
/// This struct holds all the state needed to run CashCraft:
/// - Current mode and view
/// - Settings and theme
/// - Command buffer for command mode
/// - Status messages
///
/// View-specific state will be managed by individual view components.
pub struct App {
    /// Current editing mode (Normal, Insert, Command)
    pub mode: Mode,

    /// Current view/screen
    pub view: View,

    /// Application running state
    pub running: bool,

    /// Application settings
    pub settings: Settings,

    /// Application keybindings
    pub keybindings: Keybindings,

    /// Current theme
    pub theme: Theme,

    /// Command buffer for command mode
    pub command_buffer: String,

    /// Current status message (if any)
    pub status_message: Option<StatusMessage>,

    /// Previous view (for back navigation)
    pub previous_view: Option<View>,

    /// Key sequence buffer (for multi-key commands like "gg")
    pub key_buffer: String,

    /// Whether to show the confirmation modal
    pub show_confirmation: bool,

    /// Pending action to execute on confirmation
    pub pending_action: Option<PendingAction>,

    /// Undo/Redo history
    pub history: History,
}

impl App {
    /// Create a new App with the given settings
    pub fn new(settings: Settings, keybindings: Keybindings) -> Self {
        let theme = Theme::by_name(&settings.appearance.theme).unwrap_or_default();

        Self {
            mode: Mode::Normal,
            view: View::Dashboard,
            running: true,
            settings,
            keybindings,
            theme,
            command_buffer: String::new(),
            status_message: None,
            previous_view: None,
            key_buffer: String::new(),
            show_confirmation: false,
            pending_action: None,
            history: History::new(),
        }
    }

    /// Check if the app should continue running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Set the current view
    ///
    /// Saves the previous view for back navigation.
    pub fn set_view(&mut self, view: View) {
        if self.view != view {
            self.previous_view = Some(self.view);
            self.view = view;
        }
    }

    /// Go back to the previous view
    pub fn go_back(&mut self) {
        if let Some(prev) = self.previous_view.take() {
            self.view = prev;
        }
    }

    /// Set the current mode
    ///
    /// Clears the command buffer when leaving command mode.
    pub fn set_mode(&mut self, mode: Mode) {
        if mode != Mode::Command && self.mode == Mode::Command {
            self.command_buffer.clear();
        }
        self.mode = mode;
    }

    /// Enter normal mode
    pub fn enter_normal_mode(&mut self) {
        self.set_mode(Mode::Normal);
        self.key_buffer.clear();
    }

    /// Enter insert mode
    pub fn enter_insert_mode(&mut self) {
        self.set_mode(Mode::Insert);
    }

    /// Enter command mode
    pub fn enter_command_mode(&mut self) {
        self.set_mode(Mode::Command);
        self.command_buffer.clear();
    }

    /// Set a status message
    pub fn set_status(&mut self, message: StatusMessage) {
        self.status_message = Some(message);
    }

    /// Set an info status message
    pub fn set_info(&mut self, text: impl Into<String>) {
        self.status_message = Some(StatusMessage::info(text));
    }

    /// Set a success status message
    pub fn set_success(&mut self, text: impl Into<String>) {
        self.status_message = Some(StatusMessage::success(text));
    }

    /// Set a warning status message
    pub fn set_warning(&mut self, text: impl Into<String>) {
        self.status_message = Some(StatusMessage::warning(text));
    }

    /// Set an error status message
    pub fn set_error(&mut self, text: impl Into<String>) {
        self.status_message = Some(StatusMessage::error(text));
    }

    /// Clear the status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Get the current status message text
    pub fn status_text(&self) -> Option<&str> {
        self.status_message.as_ref().map(|m| m.text.as_str())
    }

    /// Append a character to the command buffer
    pub fn push_command_char(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    /// Remove the last character from the command buffer
    pub fn pop_command_char(&mut self) {
        self.command_buffer.pop();
    }

    /// Get the current command
    pub fn command(&self) -> &str {
        &self.command_buffer
    }

    /// Clear the command buffer
    pub fn clear_command(&mut self) {
        self.command_buffer.clear();
    }

    /// Append a key to the key buffer (for multi-key sequences)
    pub fn push_key(&mut self, key: char) {
        self.key_buffer.push(key);
    }

    /// Clear the key buffer
    pub fn clear_key_buffer(&mut self) {
        self.key_buffer.clear();
    }

    /// Get the current key sequence
    pub fn key_sequence(&self) -> &str {
        &self.key_buffer
    }

    /// Update the theme
    pub fn set_theme(&mut self, theme_name: &str) {
        if let Some(theme) = Theme::by_name(theme_name) {
            self.theme = theme;
            self.settings.appearance.theme = theme_name.to_string();
            self.set_success(format!("Theme changed to {}", theme_name));
        } else {
            self.set_error(format!("Unknown theme: {}", theme_name));
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(Settings::default(), Keybindings::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_defaults() {
        let app = App::default();
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_mode_indicators() {
        assert_eq!(Mode::Normal.indicator(), "[N]");
        assert_eq!(Mode::Insert.indicator(), "[I]");
        assert_eq!(Mode::Command.indicator(), "[:]");
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(Mode::Normal.name(), "NORMAL");
        assert_eq!(Mode::Insert.name(), "INSERT");
        assert_eq!(Mode::Command.name(), "COMMAND");
    }

    #[test]
    fn test_view_defaults() {
        let app = App::default();
        assert_eq!(app.view, View::Dashboard);
    }

    #[test]
    fn test_view_shortcuts() {
        assert_eq!(View::Dashboard.shortcut(), "gh");
        assert_eq!(View::Playground.shortcut(), "gp");
        assert_eq!(View::Transactions.shortcut(), "gt");
    }

    #[test]
    fn test_view_names() {
        assert_eq!(View::Dashboard.name(), "Dashboard");
        assert_eq!(View::Playground.name(), "Playground");
        assert_eq!(View::Settings.name(), "Settings");
    }

    #[test]
    fn test_set_view_saves_previous() {
        let mut app = App::default();
        assert_eq!(app.view, View::Dashboard);
        assert!(app.previous_view.is_none());

        app.set_view(View::Transactions);
        assert_eq!(app.view, View::Transactions);
        assert_eq!(app.previous_view, Some(View::Dashboard));

        app.set_view(View::Playground);
        assert_eq!(app.view, View::Playground);
        assert_eq!(app.previous_view, Some(View::Transactions));
    }

    #[test]
    fn test_go_back() {
        let mut app = App::default();
        app.set_view(View::Transactions);
        app.set_view(View::Playground);

        app.go_back();
        assert_eq!(app.view, View::Transactions);
        assert!(app.previous_view.is_none());
    }

    #[test]
    fn test_mode_transitions() {
        let mut app = App::default();
        assert_eq!(app.mode, Mode::Normal);

        app.enter_insert_mode();
        assert_eq!(app.mode, Mode::Insert);

        app.enter_normal_mode();
        assert_eq!(app.mode, Mode::Normal);

        app.enter_command_mode();
        assert_eq!(app.mode, Mode::Command);
        assert!(app.command_buffer.is_empty());

        app.push_command_char('w');
        app.push_command_char('q');
        assert_eq!(app.command(), "wq");

        app.enter_normal_mode();
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.command_buffer.is_empty()); // Cleared on exit
    }

    #[test]
    fn test_command_buffer() {
        let mut app = App::default();
        app.enter_command_mode();

        app.push_command_char('s');
        app.push_command_char('a');
        app.push_command_char('v');
        app.push_command_char('e');
        assert_eq!(app.command(), "save");

        app.pop_command_char();
        assert_eq!(app.command(), "sav");

        app.clear_command();
        assert!(app.command().is_empty());
    }

    #[test]
    fn test_status_messages() {
        let mut app = App::default();
        assert!(app.status_message.is_none());

        app.set_info("Test info");
        assert_eq!(app.status_text(), Some("Test info"));
        assert_eq!(
            app.status_message.as_ref().unwrap().severity,
            StatusSeverity::Info
        );

        app.set_success("Saved!");
        assert_eq!(
            app.status_message.as_ref().unwrap().severity,
            StatusSeverity::Success
        );

        app.set_warning("Low balance");
        assert_eq!(
            app.status_message.as_ref().unwrap().severity,
            StatusSeverity::Warning
        );

        app.set_error("Failed to save");
        assert_eq!(
            app.status_message.as_ref().unwrap().severity,
            StatusSeverity::Error
        );

        app.clear_status();
        assert!(app.status_message.is_none());
    }

    #[test]
    fn test_quit() {
        let mut app = App::default();
        assert!(app.is_running());

        app.quit();
        assert!(!app.is_running());
    }

    #[test]
    fn test_key_buffer() {
        let mut app = App::default();

        app.push_key('g');
        assert_eq!(app.key_sequence(), "g");

        app.push_key('g');
        assert_eq!(app.key_sequence(), "gg");

        app.clear_key_buffer();
        assert!(app.key_sequence().is_empty());
    }

    #[test]
    fn test_set_theme() {
        let mut app = App::default();

        app.set_theme("nord");
        assert_eq!(app.theme.name, "Nord");
        assert_eq!(app.settings.appearance.theme, "nord");
        assert!(app.status_message.is_some());

        app.set_theme("invalid_theme");
        // Theme should not change
        assert_eq!(app.theme.name, "Nord");
        assert_eq!(
            app.status_message.as_ref().unwrap().severity,
            StatusSeverity::Error
        );
    }

    #[test]
    fn test_all_views() {
        let views = View::all();
        assert_eq!(views.len(), 8);
        assert!(views.contains(&View::Dashboard));
        assert!(views.contains(&View::Playground));
        assert!(views.contains(&View::Settings));
    }
}
