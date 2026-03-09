//! CashCraft - A Vim-powered TUI personal finance manager
//!
//! # Features
//!
//! - Vim-style navigation (hjkl, Normal/Insert/Command modes)
//! - Income and expense tracking with categories
//! - Budget management with progress visualization
//! - Playground calculator with $variable interpolation
//! - SQLite persistence with automatic migrations
//! - 10 built-in themes (6 dark, 4 light)
//! - CSV/JSON export functionality
//!
//! # Usage
//!
//! ```bash
//! # Launch TUI (default)
//! cashcraft
//!
//! # Launch with custom config
//! cashcraft --config ~/.config/cashcraft/custom.toml
//!
//! # Quick add transaction
//! cashcraft add transaction
//!
//! # Export data
//! cashcraft export csv
//!
//! # Open playground directly
//! cashcraft playground
//! ```

// Module declarations
pub mod app;
pub mod config;
pub mod domain;
pub mod error;
pub mod repository;
pub mod services;
pub mod ui;
pub mod utils;

// Re-export common types
pub use error::{CashCraftError, Result};

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

/// CashCraft - A Vim-powered TUI personal finance manager
#[derive(Parser)]
#[command(name = "cashcraft")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Use custom config file
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Override theme (e.g., dracula, nord, tokyo-night)
    #[arg(short, long)]
    pub theme: Option<String>,

    /// Use custom data directory
    #[arg(short, long, value_name = "PATH")]
    pub data: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable animations
    #[arg(long)]
    pub no_animations: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available commands
#[derive(Subcommand)]
pub enum Command {
    /// Launch the TUI (default when no command specified)
    Tui,

    /// Quick add an item (income, expense, or transaction)
    Add {
        /// Type of item to add: income, expense, transaction
        item_type: String,
    },

    /// List items
    List {
        /// Type of items to list: income, expenses, transactions, budgets
        item_type: String,
    },

    /// Open the playground calculator directly
    Playground,

    /// Export data to a file
    Export {
        /// Export format: csv, json
        format: String,

        /// Output file path (optional, defaults to exports directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import data from a file
    Import {
        /// Path to the file to import
        file: PathBuf,
    },

    /// Create a backup of the database
    Backup {
        /// Output path (optional, defaults to backups directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Restore from a backup file
    Restore {
        /// Path to the backup file
        file: PathBuf,
    },

    /// Edit configuration (opens $EDITOR or default config path)
    Config {
        /// Print config path instead of opening editor
        #[arg(long)]
        path: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    info!("CashCraft v{}", env!("CARGO_PKG_VERSION"));

    // Ensure directories exist
    if let Err(e) = config::ensure_directories() {
        eprintln!("Failed to create directories: {}", e);
        std::process::exit(1);
    }

    // Load configuration
    let config_path = cli
        .config
        .unwrap_or_else(config::settings::Settings::default_path);
    let mut settings = match config::Settings::load(&config_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load config from {:?}: {}", config_path, e);
            eprintln!("Using default settings.");
            config::Settings::default()
        }
    };

    let keybindings = match config::Keybindings::load(&config::Keybindings::default_path()) {
        Ok(kb) => kb,
        Err(e) => {
            eprintln!("Failed to load keybindings: {}", e);
            config::Keybindings::default()
        }
    };

    // Apply CLI overrides
    if let Some(theme) = &cli.theme {
        settings.appearance.theme = theme.clone();
    }
    if cli.no_animations {
        settings.appearance.animations_enabled = false;
    }

    // Execute command
    match cli.command {
        None | Some(Command::Tui) => {
            run_tui(settings, keybindings);
        }
        Some(Command::Add { item_type }) => {
            run_add(&item_type);
        }
        Some(Command::List { item_type }) => {
            run_list(&item_type);
        }
        Some(Command::Playground) => {
            run_playground(settings, keybindings);
        }
        Some(Command::Export { format, output }) => {
            run_export(&format, output);
        }
        Some(Command::Import { file }) => {
            run_import(&file);
        }
        Some(Command::Backup { output }) => {
            run_backup(output);
        }
        Some(Command::Restore { file }) => {
            run_restore(&file);
        }
        Some(Command::Config { path }) => {
            run_config(path);
        }
    }
}

/// Initialize the logging system
fn init_logging(verbose: bool) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

/// Run the main TUI application
fn run_tui(settings: config::Settings, keybindings: config::Keybindings) {
    match ui::tui::TuiRunner::new(settings, keybindings) {
        Ok(mut runner) => {
            if let Err(e) = runner.run() {
                // Terminal will be restored by Drop
                eprintln!("TUI error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize TUI: {}", e);
            std::process::exit(1);
        }
    }
}

/// Quick add an item
fn run_add(item_type: &str) {
    // TODO: Implement quick add
    match item_type.to_lowercase().as_str() {
        "income" => println!("Quick add income - Coming soon!"),
        "expense" => println!("Quick add expense - Coming soon!"),
        "transaction" => println!("Quick add transaction - Coming soon!"),
        _ => {
            eprintln!(
                "Unknown item type: {}. Use: income, expense, transaction",
                item_type
            );
            std::process::exit(1);
        }
    }
}

/// List items
fn run_list(item_type: &str) {
    // TODO: Implement list
    match item_type.to_lowercase().as_str() {
        "income" => println!("Income sources - Coming soon!"),
        "expenses" => println!("Expenses - Coming soon!"),
        "transactions" => println!("Transactions - Coming soon!"),
        "budgets" => println!("Budgets - Coming soon!"),
        _ => {
            eprintln!(
                "Unknown item type: {}. Use: income, expenses, transactions, budgets",
                item_type
            );
            std::process::exit(1);
        }
    }
}

/// Open playground directly
fn run_playground(settings: config::Settings, keybindings: config::Keybindings) {
    let mut _app = app::App::new(settings, keybindings);
    _app.set_view(app::View::Playground);

    // TODO: Implement playground mode
    println!("Playground Calculator - Coming soon!");
    println!("Use expressions like: $salary - $rent + 1000");
}

/// Export data
fn run_export(format: &str, output: Option<PathBuf>) {
    // TODO: Implement export
    let output_path = output.unwrap_or_else(|| {
        config::exports_dir().join(format!(
            "cashcraft_export_{}.{}",
            chrono::Local::now().format("%Y%m%d_%H%M%S"),
            format
        ))
    });

    match format.to_lowercase().as_str() {
        "csv" => println!("Exporting to CSV: {:?} - Coming soon!", output_path),
        "json" => println!("Exporting to JSON: {:?} - Coming soon!", output_path),
        _ => {
            eprintln!("Unknown format: {}. Use: csv, json", format);
            std::process::exit(1);
        }
    }
}

/// Import data
fn run_import(file: &PathBuf) {
    // TODO: Implement import
    if !file.exists() {
        eprintln!("File not found: {:?}", file);
        std::process::exit(1);
    }
    println!("Importing from {:?} - Coming soon!", file);
}

/// Create backup
fn run_backup(output: Option<PathBuf>) {
    // TODO: Implement backup
    let output_path = output.unwrap_or_else(|| {
        config::backups_dir().join(format!(
            "cashcraft_backup_{}.db",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        ))
    });
    println!("Creating backup at {:?} - Coming soon!", output_path);
}

/// Restore from backup
fn run_restore(file: &PathBuf) {
    // TODO: Implement restore
    if !file.exists() {
        eprintln!("Backup file not found: {:?}", file);
        std::process::exit(1);
    }
    println!("Restoring from {:?} - Coming soon!", file);
}

/// Edit or show config
fn run_config(show_path: bool) {
    let config_path = config::settings::Settings::default_path();

    if show_path {
        println!("{}", config_path.display());
        return;
    }

    // Try to open with $EDITOR
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    // Ensure config file exists with defaults
    if !config_path.exists() {
        if let Err(e) = config::Settings::default().save(&config_path) {
            eprintln!("Failed to create default config: {}", e);
            std::process::exit(1);
        }
    }

    // Open editor
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("Editor exited with status: {}", s);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to open editor '{}': {}", editor, e);
            eprintln!("Config file is at: {}", config_path.display());
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // Test default (no args)
        let cli = Cli::parse_from(["cashcraft"]);
        assert!(cli.config.is_none());
        assert!(cli.theme.is_none());
        assert!(!cli.verbose);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_with_options() {
        let cli = Cli::parse_from([
            "cashcraft",
            "--config",
            "/path/to/config.toml",
            "--theme",
            "nord",
            "--verbose",
            "--no-animations",
        ]);

        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.toml")));
        assert_eq!(cli.theme, Some("nord".to_string()));
        assert!(cli.verbose);
        assert!(cli.no_animations);
    }

    #[test]
    fn test_cli_tui_command() {
        let cli = Cli::parse_from(["cashcraft", "tui"]);
        assert!(matches!(cli.command, Some(Command::Tui)));
    }

    #[test]
    fn test_cli_add_command() {
        let cli = Cli::parse_from(["cashcraft", "add", "income"]);
        match cli.command {
            Some(Command::Add { item_type }) => assert_eq!(item_type, "income"),
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_cli_list_command() {
        let cli = Cli::parse_from(["cashcraft", "list", "transactions"]);
        match cli.command {
            Some(Command::List { item_type }) => assert_eq!(item_type, "transactions"),
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_cli_playground_command() {
        let cli = Cli::parse_from(["cashcraft", "playground"]);
        assert!(matches!(cli.command, Some(Command::Playground)));
    }

    #[test]
    fn test_cli_export_command() {
        let cli = Cli::parse_from(["cashcraft", "export", "csv", "--output", "/tmp/export.csv"]);
        match cli.command {
            Some(Command::Export { format, output }) => {
                assert_eq!(format, "csv");
                assert_eq!(output, Some(PathBuf::from("/tmp/export.csv")));
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn test_cli_import_command() {
        let cli = Cli::parse_from(["cashcraft", "import", "/tmp/data.csv"]);
        match cli.command {
            Some(Command::Import { file }) => {
                assert_eq!(file, PathBuf::from("/tmp/data.csv"));
            }
            _ => panic!("Expected Import command"),
        }
    }

    #[test]
    fn test_cli_backup_command() {
        let cli = Cli::parse_from(["cashcraft", "backup"]);
        match cli.command {
            Some(Command::Backup { output }) => {
                assert!(output.is_none());
            }
            _ => panic!("Expected Backup command"),
        }
    }

    #[test]
    fn test_cli_restore_command() {
        let cli = Cli::parse_from(["cashcraft", "restore", "/tmp/backup.db"]);
        match cli.command {
            Some(Command::Restore { file }) => {
                assert_eq!(file, PathBuf::from("/tmp/backup.db"));
            }
            _ => panic!("Expected Restore command"),
        }
    }

    #[test]
    fn test_cli_config_command() {
        let cli = Cli::parse_from(["cashcraft", "config", "--path"]);
        match cli.command {
            Some(Command::Config { path }) => {
                assert!(path);
            }
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_cli_help() {
        // Just verify the command builds without panicking
        Cli::command().debug_assert();
    }

    #[test]
    fn test_short_options() {
        let cli = Cli::parse_from([
            "cashcraft",
            "-c",
            "/path/config.toml",
            "-t",
            "dracula",
            "-v",
        ]);

        assert_eq!(cli.config, Some(PathBuf::from("/path/config.toml")));
        assert_eq!(cli.theme, Some("dracula".to_string()));
        assert!(cli.verbose);
    }
}
