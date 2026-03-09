//! Configuration management
//!
//! Handles settings, keybindings, and directory paths for CashCraft.
//!
//! # File Structure
//!
//! ```text
//! ~/.config/cashcraft/          # or platform-specific config dir
//! ├── settings.toml             # Main configuration
//! ├── keybindings.toml          # Custom keybindings
//! └── themes/                   # Custom themes
//!     └── my_theme.toml
//!
//! ~/.local/share/cashcraft/     # or platform-specific data dir
//! ├── cashcraft.db              # SQLite database
//! ├── backups/                  # Automatic backups
//! ├── exports/                  # Exported files
//! └── playground/               # Saved playground sessions
//!     └── sessions/
//! ```

pub mod keybindings;
pub mod settings;

pub use keybindings::{Key, Keybindings};
pub use settings::Settings;

use directories::ProjectDirs;
use std::path::PathBuf;

/// Application identifier for directory lookup
const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "cashcraft";
const APPLICATION: &str = "CashCraft";

/// Get the application config directory
///
/// Returns the platform-specific config directory:
/// - Linux: `~/.config/cashcraft`
/// - macOS: `~/Library/Application Support/com.cashcraft.CashCraft`
/// - Windows: `C:\Users\<User>\AppData\Roaming\cashcraft\CashCraft\config`
pub fn config_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        proj_dirs.config_dir().to_path_buf()
    } else {
        // Fallback to ~/.config/cashcraft
        dirs_fallback().join("config")
    }
}

/// Get the application data directory
///
/// Returns the platform-specific data directory:
/// - Linux: `~/.local/share/cashcraft`
/// - macOS: `~/Library/Application Support/com.cashcraft.CashCraft`
/// - Windows: `C:\Users\<User>\AppData\Roaming\cashcraft\CashCraft\data`
pub fn data_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        proj_dirs.data_dir().to_path_buf()
    } else {
        // Fallback to ~/.local/share/cashcraft
        dirs_fallback().join("data")
    }
}

/// Get the cache directory
///
/// Returns the platform-specific cache directory:
/// - Linux: `~/.cache/cashcraft`
/// - macOS: `~/Library/Caches/com.cashcraft.CashCraft`
/// - Windows: `C:\Users\<User>\AppData\Local\cashcraft\CashCraft\cache`
pub fn cache_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        proj_dirs.cache_dir().to_path_buf()
    } else {
        dirs_fallback().join("cache")
    }
}

/// Get the logs directory
pub fn logs_dir() -> PathBuf {
    data_dir().join("logs")
}

/// Get the backups directory
pub fn backups_dir() -> PathBuf {
    data_dir().join("backups")
}

/// Get the exports directory
pub fn exports_dir() -> PathBuf {
    data_dir().join("exports")
}

/// Get the playground sessions directory
pub fn playground_dir() -> PathBuf {
    data_dir().join("playground").join("sessions")
}

/// Get the themes directory
pub fn themes_dir() -> PathBuf {
    config_dir().join("themes")
}

/// Get the database file path
pub fn database_path() -> PathBuf {
    data_dir().join("cashcraft.db")
}

/// Fallback directory when ProjectDirs fails
fn dirs_fallback() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".cashcraft")
    } else {
        PathBuf::from(".cashcraft")
    }
}

/// Ensure all required directories exist
///
/// Creates the directory structure if it doesn't exist.
pub fn ensure_directories() -> anyhow::Result<()> {
    std::fs::create_dir_all(config_dir())?;
    std::fs::create_dir_all(data_dir())?;
    std::fs::create_dir_all(cache_dir())?;
    std::fs::create_dir_all(logs_dir())?;
    std::fs::create_dir_all(backups_dir())?;
    std::fs::create_dir_all(exports_dir())?;
    std::fs::create_dir_all(playground_dir())?;
    std::fs::create_dir_all(themes_dir())?;
    Ok(())
}

/// Load all configuration
///
/// Loads settings and keybindings from their default paths.
pub fn load_config() -> anyhow::Result<(Settings, Keybindings)> {
    let settings = Settings::load(&Settings::default_path())?;
    let keybindings = Keybindings::load(&Keybindings::default_path())?;
    Ok((settings, keybindings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_not_empty() {
        let dir = config_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_data_dir_not_empty() {
        let dir = data_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_database_path() {
        let path = database_path();
        assert!(path.ends_with("cashcraft.db"));
    }

    #[test]
    fn test_subdirectories() {
        assert!(logs_dir().ends_with("logs"));
        assert!(backups_dir().ends_with("backups"));
        assert!(exports_dir().ends_with("exports"));
        assert!(themes_dir().ends_with("themes"));
    }

    #[test]
    fn test_playground_dir() {
        let dir = playground_dir();
        assert!(dir.to_string_lossy().contains("playground"));
        assert!(dir.ends_with("sessions"));
    }
}
