//! Vim-style keybindings configuration
//!
//! Provides a customizable keybinding system with:
//! - Global shortcuts (quit, save, help, search)
//! - Navigation keys (hjkl, gg, G, Ctrl+d/u)
//! - View-specific keybindings
//! - Leader key sequences
//! - Full customization via TOML config

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete keybinding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    pub global: GlobalKeybindings,
    pub navigation: NavigationKeybindings,
    pub views: ViewKeybindings,
    pub leader: LeaderKeybindings,
    /// Custom user-defined keybindings
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// Global keybindings available in all modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalKeybindings {
    /// Quit application
    pub quit: String,
    /// Force quit without saving
    pub force_quit: String,
    /// Save current changes
    pub save: String,
    /// Show help
    pub help: String,
    /// Open search
    pub search: String,
    /// Enter command mode
    pub command: String,
    /// Cancel/return to normal mode
    pub escape: String,
    /// Undo last action
    pub undo: String,
    /// Redo last undone action
    pub redo: String,
    /// Next panel/tab
    pub next_panel: String,
    /// Previous panel/tab
    pub prev_panel: String,
    /// Select/toggle item
    pub select: String,
    /// Confirm action
    pub confirm: String,
}

/// Navigation keybindings for movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationKeybindings {
    /// Move up
    pub up: String,
    /// Move down
    pub down: String,
    /// Move left
    pub left: String,
    /// Move right
    pub right: String,
    /// Go to top/first item
    pub top: String,
    /// Go to bottom/last item
    pub bottom: String,
    /// Half page down
    pub half_page_down: String,
    /// Half page up
    pub half_page_up: String,
    /// Full page down
    pub page_down: String,
    /// Full page up
    pub page_up: String,
    /// Next word/item
    pub next_word: String,
    /// Previous word/item
    pub prev_word: String,
    /// Start of line
    pub line_start: String,
    /// End of line
    pub line_end: String,
    /// Previous section
    pub prev_section: String,
    /// Next section
    pub next_section: String,
}

/// View navigation keybindings (go to view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewKeybindings {
    /// Go to dashboard/home
    pub home: String,
    /// Go to transactions
    pub transactions: String,
    /// Go to playground
    pub playground: String,
    /// Go to charts/graphs
    pub charts: String,
    /// Go to income
    pub income: String,
    /// Go to expenses
    pub expenses: String,
    /// Go to budget
    pub budget: String,
    /// Go to settings
    pub settings: String,
    /// Go to current month
    pub current_month: String,
    /// Open month selector
    pub month_selector: String,
    /// Go to calendar view
    pub calendar: String,
    /// Go to reports
    pub reports: String,
}

/// Leader key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderKeybindings {
    /// The leader key (e.g., "Space", "\\")
    pub key: String,
    /// Timeout in milliseconds for leader sequences
    pub timeout_ms: u64,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            global: GlobalKeybindings::default(),
            navigation: NavigationKeybindings::default(),
            views: ViewKeybindings::default(),
            leader: LeaderKeybindings::default(),
            custom: HashMap::new(),
        }
    }
}

impl Default for GlobalKeybindings {
    fn default() -> Self {
        Self {
            quit: "q".into(),
            force_quit: "Q".into(),
            save: "C-s".into(),
            help: "?".into(),
            search: "/".into(),
            command: ":".into(),
            escape: "Esc".into(),
            undo: "C-z".into(),
            redo: "C-r".into(),
            next_panel: "Tab".into(),
            prev_panel: "S-Tab".into(),
            select: "Space".into(),
            confirm: "Enter".into(),
        }
    }
}

impl Default for NavigationKeybindings {
    fn default() -> Self {
        Self {
            up: "k".into(),
            down: "j".into(),
            left: "h".into(),
            right: "l".into(),
            top: "gg".into(),
            bottom: "G".into(),
            half_page_down: "C-d".into(),
            half_page_up: "C-u".into(),
            page_down: "C-f".into(),
            page_up: "C-b".into(),
            next_word: "w".into(),
            prev_word: "b".into(),
            line_start: "0".into(),
            line_end: "$".into(),
            prev_section: "{".into(),
            next_section: "}".into(),
        }
    }
}

impl Default for ViewKeybindings {
    fn default() -> Self {
        Self {
            home: "gh".into(),
            transactions: "gt".into(),
            playground: "gp".into(),
            charts: "gg".into(),
            income: "gi".into(),
            expenses: "ge".into(),
            budget: "gb".into(),
            settings: "gs".into(),
            current_month: "gm".into(),
            month_selector: "gM".into(),
            calendar: "gc".into(),
            reports: "gr".into(),
        }
    }
}

impl Default for LeaderKeybindings {
    fn default() -> Self {
        Self {
            key: "Space".into(),
            timeout_ms: 1000,
        }
    }
}

impl Keybindings {
    /// Load keybindings from a TOML file
    ///
    /// If the file doesn't exist, returns default keybindings.
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let keybindings: Keybindings = toml::from_str(&content)?;
            Ok(keybindings)
        } else {
            Ok(Self::default())
        }
    }

    /// Save keybindings to a TOML file
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default keybindings file path
    pub fn default_path() -> PathBuf {
        crate::config::config_dir().join("keybindings.toml")
    }

    /// Get a keybinding by action name
    pub fn get(&self, action: &str) -> Option<&str> {
        // Check custom keybindings first
        if let Some(key) = self.custom.get(action) {
            return Some(key.as_str());
        }

        // Check standard keybindings
        match action {
            // Global
            "quit" => Some(&self.global.quit),
            "force_quit" => Some(&self.global.force_quit),
            "save" => Some(&self.global.save),
            "help" => Some(&self.global.help),
            "search" => Some(&self.global.search),
            "command" => Some(&self.global.command),
            "escape" => Some(&self.global.escape),
            "undo" => Some(&self.global.undo),
            "redo" => Some(&self.global.redo),
            "next_panel" => Some(&self.global.next_panel),
            "prev_panel" => Some(&self.global.prev_panel),
            "select" => Some(&self.global.select),
            "confirm" => Some(&self.global.confirm),

            // Navigation
            "up" => Some(&self.navigation.up),
            "down" => Some(&self.navigation.down),
            "left" => Some(&self.navigation.left),
            "right" => Some(&self.navigation.right),
            "top" => Some(&self.navigation.top),
            "bottom" => Some(&self.navigation.bottom),
            "half_page_down" => Some(&self.navigation.half_page_down),
            "half_page_up" => Some(&self.navigation.half_page_up),
            "page_down" => Some(&self.navigation.page_down),
            "page_up" => Some(&self.navigation.page_up),
            "next_word" => Some(&self.navigation.next_word),
            "prev_word" => Some(&self.navigation.prev_word),
            "line_start" => Some(&self.navigation.line_start),
            "line_end" => Some(&self.navigation.line_end),
            "prev_section" => Some(&self.navigation.prev_section),
            "next_section" => Some(&self.navigation.next_section),

            // Views
            "home" | "dashboard" => Some(&self.views.home),
            "transactions" => Some(&self.views.transactions),
            "playground" => Some(&self.views.playground),
            "charts" => Some(&self.views.charts),
            "income" => Some(&self.views.income),
            "expenses" => Some(&self.views.expenses),
            "budget" => Some(&self.views.budget),
            "settings" => Some(&self.views.settings),
            "current_month" => Some(&self.views.current_month),
            "month_selector" => Some(&self.views.month_selector),
            "calendar" => Some(&self.views.calendar),
            "reports" => Some(&self.views.reports),

            _ => None,
        }
    }

    /// Add or update a custom keybinding
    pub fn set_custom(&mut self, action: String, key: String) {
        self.custom.insert(action, key);
    }

    /// Remove a custom keybinding
    pub fn remove_custom(&mut self, action: &str) -> Option<String> {
        self.custom.remove(action)
    }
}

/// Key representation for parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// The base key (e.g., "j", "Enter", "Space")
    pub key: String,
    /// Ctrl modifier
    pub ctrl: bool,
    /// Shift modifier
    pub shift: bool,
    /// Alt modifier
    pub alt: bool,
}

impl Key {
    /// Parse a key string into a Key struct
    ///
    /// Format: `[C-][S-][A-]<key>`
    /// Examples: "j", "C-s", "S-Tab", "C-S-r"
    pub fn parse(s: &str) -> Option<Self> {
        let mut key = Self {
            key: String::new(),
            ctrl: false,
            shift: false,
            alt: false,
        };

        let parts: Vec<&str> = s.split('-').collect();
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part is the key
                key.key = (*part).to_string();
            } else {
                // Modifier
                match *part {
                    "C" => key.ctrl = true,
                    "S" => key.shift = true,
                    "A" => key.alt = true,
                    _ => return None, // Invalid modifier
                }
            }
        }

        if key.key.is_empty() {
            None
        } else {
            Some(key)
        }
    }

    /// Check if this key matches a crossterm KeyEvent
    pub fn matches_event(&self, event: &crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        let modifiers = event.modifiers;
        let ctrl_match = self.ctrl == modifiers.contains(KeyModifiers::CONTROL);
        let shift_match = self.shift == modifiers.contains(KeyModifiers::SHIFT);
        let alt_match = self.alt == modifiers.contains(KeyModifiers::ALT);

        if !ctrl_match || !shift_match || !alt_match {
            return false;
        }

        // Check key code
        match event.code {
            KeyCode::Char(c) => {
                // Handle single character keys
                if self.key.len() == 1 {
                    self.key.chars().next() == Some(c)
                } else {
                    false
                }
            }
            KeyCode::Enter => self.key == "Enter",
            KeyCode::Esc => self.key == "Esc",
            KeyCode::Tab => self.key == "Tab",
            KeyCode::Backspace => self.key == "Backspace",
            KeyCode::Delete => self.key == "Delete",
            KeyCode::Up => self.key == "Up",
            KeyCode::Down => self.key == "Down",
            KeyCode::Left => self.key == "Left",
            KeyCode::Right => self.key == "Right",
            KeyCode::Home => self.key == "Home",
            KeyCode::End => self.key == "End",
            KeyCode::PageUp => self.key == "PageUp",
            KeyCode::PageDown => self.key == "PageDown",
            KeyCode::F(n) => self.key == format!("F{}", n),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_keybindings() {
        let kb = Keybindings::default();

        // Global
        assert_eq!(kb.global.quit, "q");
        assert_eq!(kb.global.save, "C-s");
        assert_eq!(kb.global.help, "?");

        // Navigation
        assert_eq!(kb.navigation.up, "k");
        assert_eq!(kb.navigation.down, "j");
        assert_eq!(kb.navigation.left, "h");
        assert_eq!(kb.navigation.right, "l");
        assert_eq!(kb.navigation.top, "gg");
        assert_eq!(kb.navigation.bottom, "G");

        // Views
        assert_eq!(kb.views.home, "gh");
        assert_eq!(kb.views.playground, "gp");
        assert_eq!(kb.views.transactions, "gt");

        // Leader
        assert_eq!(kb.leader.key, "Space");
        assert_eq!(kb.leader.timeout_ms, 1000);
    }

    #[test]
    fn test_get_keybinding() {
        let kb = Keybindings::default();

        assert_eq!(kb.get("quit"), Some("q"));
        assert_eq!(kb.get("save"), Some("C-s"));
        assert_eq!(kb.get("up"), Some("k"));
        assert_eq!(kb.get("home"), Some("gh"));
        assert_eq!(kb.get("dashboard"), Some("gh")); // Alias
        assert_eq!(kb.get("nonexistent"), None);
    }

    #[test]
    fn test_custom_keybindings() {
        let mut kb = Keybindings::default();

        // Add custom
        kb.set_custom("my_action".into(), "C-m".into());
        assert_eq!(kb.get("my_action"), Some("C-m"));

        // Custom overrides don't affect standard
        kb.set_custom("quit".into(), "C-q".into());
        assert_eq!(kb.get("quit"), Some("C-q")); // Custom takes precedence

        // Remove custom
        kb.remove_custom("my_action");
        assert_eq!(kb.get("my_action"), None);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("keybindings.toml");

        let mut kb = Keybindings::default();
        kb.global.quit = "C-q".into();
        kb.navigation.up = "w".into();
        kb.set_custom("custom_action".into(), "C-x".into());

        // Save
        kb.save(&path).unwrap();
        assert!(path.exists());

        // Load
        let loaded = Keybindings::load(&path).unwrap();
        assert_eq!(loaded.global.quit, "C-q");
        assert_eq!(loaded.navigation.up, "w");
        assert_eq!(loaded.custom.get("custom_action"), Some(&"C-x".to_string()));
    }

    #[test]
    fn test_key_parse() {
        // Simple key
        let key = Key::parse("j").unwrap();
        assert_eq!(key.key, "j");
        assert!(!key.ctrl);
        assert!(!key.shift);
        assert!(!key.alt);

        // Ctrl modifier
        let key = Key::parse("C-s").unwrap();
        assert_eq!(key.key, "s");
        assert!(key.ctrl);
        assert!(!key.shift);

        // Shift modifier
        let key = Key::parse("S-Tab").unwrap();
        assert_eq!(key.key, "Tab");
        assert!(!key.ctrl);
        assert!(key.shift);

        // Multiple modifiers
        let key = Key::parse("C-S-r").unwrap();
        assert_eq!(key.key, "r");
        assert!(key.ctrl);
        assert!(key.shift);

        // All modifiers
        let key = Key::parse("C-S-A-x").unwrap();
        assert_eq!(key.key, "x");
        assert!(key.ctrl);
        assert!(key.shift);
        assert!(key.alt);

        // Special keys
        let key = Key::parse("Enter").unwrap();
        assert_eq!(key.key, "Enter");

        let key = Key::parse("Esc").unwrap();
        assert_eq!(key.key, "Esc");
    }

    #[test]
    fn test_serialization() {
        let kb = Keybindings::default();
        let toml_str = toml::to_string_pretty(&kb).unwrap();

        assert!(toml_str.contains("[global]"));
        assert!(toml_str.contains("[navigation]"));
        assert!(toml_str.contains("[views]"));
        assert!(toml_str.contains("[leader]"));
        assert!(toml_str.contains("quit = \"q\""));
        assert!(toml_str.contains("up = \"k\""));
    }
}
