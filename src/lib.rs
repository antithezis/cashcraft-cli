//! CashCraft - A Vim-powered TUI personal finance manager
//!
//! CashCraft provides terminal-based personal finance management with:
//! - Vim-style navigation and keybindings
//! - Income and expense tracking
//! - Budget management
//! - Playground calculator with variable interpolation
//! - Rich ASCII/Unicode visualizations

pub mod app;
pub mod config;
pub mod domain;
pub mod error;
pub mod repository;
pub mod services;
pub mod ui;
pub mod utils;

pub use error::{CashCraftError, Result};
