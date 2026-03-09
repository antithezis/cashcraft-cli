//! Application settings
//!
//! Comprehensive settings management for CashCraft with support for:
//! - General preferences (language, currency, date format)
//! - Appearance settings (theme, animations, borders)
//! - Navigation preferences (vim mode, leader key)
//! - Playground configuration
//! - Data management (backups, auto-save)
//! - Notifications

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub navigation: NavigationSettings,
    pub playground: PlaygroundSettings,
    pub data: DataSettings,
    pub notifications: NotificationSettings,
}

/// General settings for locale and formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// Language code (e.g., "en", "es", "pt")
    pub language: String,
    /// Currency code (e.g., "USD", "EUR")
    pub currency: String,
    /// Currency symbol (e.g., "$", "EUR")
    pub currency_symbol: String,
    /// Position of currency symbol: "before" or "after"
    pub currency_position: String,
    /// Decimal separator character
    pub decimal_separator: char,
    /// Thousands separator character
    pub thousands_separator: char,
    /// Date format string (strftime format)
    pub date_format: String,
    /// First day of week: "monday" or "sunday"
    pub first_day_of_week: String,
}

/// Appearance settings for theme and visual effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// Theme name (e.g., "dracula", "nord", "tokyo-night")
    pub theme: String,
    /// Enable animations
    pub animations_enabled: bool,
    /// Animation speed: "slow", "normal", "fast", "instant"
    pub animation_speed: String,
    /// Reduced motion for accessibility
    pub reduced_motion: bool,
    /// Use Unicode box drawing characters for borders
    pub unicode_borders: bool,
    /// Show icons in the UI
    pub show_icons: bool,
    /// Compact mode for smaller terminals
    pub compact_mode: bool,
}

/// Navigation settings for vim-style controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationSettings {
    /// Enable vim-style navigation
    pub vim_mode: bool,
    /// Leader key for shortcuts (e.g., "Space")
    pub leader_key: String,
    /// Number of lines to scroll with Ctrl+d/u
    pub scroll_amount: usize,
}

/// Playground calculator settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaygroundSettings {
    /// Auto-evaluate expressions on Enter
    pub auto_evaluate: bool,
    /// Show global variables panel
    pub show_global_vars: bool,
    /// Default decimal places for results
    pub decimal_places: usize,
    /// Save calculation history
    pub save_history: bool,
    /// Maximum history entries to keep
    pub max_history: usize,
}

/// Data management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSettings {
    /// Enable automatic backups
    pub auto_backup: bool,
    /// Backup frequency: "daily", "weekly", "monthly"
    pub backup_frequency: String,
    /// Days to keep backups
    pub backup_retention: usize,
    /// Enable auto-save
    pub auto_save: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable notifications
    pub enabled: bool,
    /// Show budget warnings
    pub budget_warnings: bool,
    /// Budget warning threshold percentage
    pub budget_warning_threshold: u8,
    /// Show upcoming bills reminders
    pub upcoming_bills: bool,
    /// Days before due date to show reminder
    pub bills_reminder_days: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            appearance: AppearanceSettings::default(),
            navigation: NavigationSettings::default(),
            playground: PlaygroundSettings::default(),
            data: DataSettings::default(),
            notifications: NotificationSettings::default(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            currency: "USD".into(),
            currency_symbol: "$".into(),
            currency_position: "before".into(),
            decimal_separator: '.',
            thousands_separator: ',',
            date_format: "%m/%d/%Y".into(),
            first_day_of_week: "monday".into(),
        }
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "dracula".into(),
            animations_enabled: true,
            animation_speed: "normal".into(),
            reduced_motion: false,
            unicode_borders: true,
            show_icons: true,
            compact_mode: false,
        }
    }
}

impl Default for NavigationSettings {
    fn default() -> Self {
        Self {
            vim_mode: true,
            leader_key: "Space".into(),
            scroll_amount: 10,
        }
    }
}

impl Default for PlaygroundSettings {
    fn default() -> Self {
        Self {
            auto_evaluate: true,
            show_global_vars: true,
            decimal_places: 2,
            save_history: true,
            max_history: 100,
        }
    }
}

impl Default for DataSettings {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_frequency: "daily".into(),
            backup_retention: 30,
            auto_save: true,
            auto_save_interval: 60,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            budget_warnings: true,
            budget_warning_threshold: 80,
            upcoming_bills: true,
            bills_reminder_days: 3,
        }
    }
}

impl Settings {
    /// Load settings from a TOML file
    ///
    /// If the file doesn't exist, returns default settings.
    /// If the file exists but is invalid, returns an error.
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let settings: Settings = toml::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(Self::default())
        }
    }

    /// Save settings to a TOML file
    ///
    /// Creates parent directories if they don't exist.
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default settings file path
    pub fn default_path() -> PathBuf {
        crate::config::config_dir().join("settings.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        // General
        assert_eq!(settings.general.language, "en");
        assert_eq!(settings.general.currency, "USD");
        assert_eq!(settings.general.currency_symbol, "$");
        assert_eq!(settings.general.currency_position, "before");
        assert_eq!(settings.general.decimal_separator, '.');
        assert_eq!(settings.general.thousands_separator, ',');
        assert_eq!(settings.general.date_format, "%m/%d/%Y");
        assert_eq!(settings.general.first_day_of_week, "monday");

        // Appearance
        assert_eq!(settings.appearance.theme, "dracula");
        assert!(settings.appearance.animations_enabled);
        assert_eq!(settings.appearance.animation_speed, "normal");
        assert!(!settings.appearance.reduced_motion);
        assert!(settings.appearance.unicode_borders);
        assert!(settings.appearance.show_icons);
        assert!(!settings.appearance.compact_mode);

        // Navigation
        assert!(settings.navigation.vim_mode);
        assert_eq!(settings.navigation.leader_key, "Space");
        assert_eq!(settings.navigation.scroll_amount, 10);

        // Playground
        assert!(settings.playground.auto_evaluate);
        assert!(settings.playground.show_global_vars);
        assert_eq!(settings.playground.decimal_places, 2);
        assert!(settings.playground.save_history);
        assert_eq!(settings.playground.max_history, 100);

        // Data
        assert!(settings.data.auto_backup);
        assert_eq!(settings.data.backup_frequency, "daily");
        assert_eq!(settings.data.backup_retention, 30);
        assert!(settings.data.auto_save);
        assert_eq!(settings.data.auto_save_interval, 60);

        // Notifications
        assert!(settings.notifications.enabled);
        assert!(settings.notifications.budget_warnings);
        assert_eq!(settings.notifications.budget_warning_threshold, 80);
        assert!(settings.notifications.upcoming_bills);
        assert_eq!(settings.notifications.bills_reminder_days, 3);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.toml");

        let mut settings = Settings::default();
        settings.general.language = "es".into();
        settings.appearance.theme = "nord".into();
        settings.navigation.scroll_amount = 20;

        // Save
        settings.save(&path).unwrap();
        assert!(path.exists());

        // Load
        let loaded = Settings::load(&path).unwrap();
        assert_eq!(loaded.general.language, "es");
        assert_eq!(loaded.appearance.theme, "nord");
        assert_eq!(loaded.navigation.scroll_amount, 20);
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let path = PathBuf::from("/nonexistent/path/settings.toml");
        let settings = Settings::load(&path).unwrap();
        assert_eq!(settings.general.language, "en");
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings::default();
        let toml_str = toml::to_string_pretty(&settings).unwrap();

        // Verify key sections exist
        assert!(toml_str.contains("[general]"));
        assert!(toml_str.contains("[appearance]"));
        assert!(toml_str.contains("[navigation]"));
        assert!(toml_str.contains("[playground]"));
        assert!(toml_str.contains("[data]"));
        assert!(toml_str.contains("[notifications]"));

        // Verify some values
        assert!(toml_str.contains("language = \"en\""));
        assert!(toml_str.contains("theme = \"dracula\""));
        assert!(toml_str.contains("vim_mode = true"));
    }
}
