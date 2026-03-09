//! Custom widgets for CashCraft TUI
//!
//! Reusable UI components built on Ratatui, including:
//! - VimTable: Table with Vim-style navigation
//! - TextInput: Text input with cursor and modes
//! - ProgressBar: Budget utilization bars
//! - Charts: Bar, line, sparkline, pie charts

pub mod chart;
pub mod input;
pub mod progress;
pub mod table;

// Table widgets
pub use table::{Alignment, ColumnWidth, SimpleTable, TableColumn, TableRow, TableState, VimTable};

// Input widgets
pub use input::{InputState, MultiLineInputState, TextInput};

// Progress widgets
pub use progress::{BudgetProgress, CircularProgress, MiniProgress, ProgressBar, ProgressStyle};

// Chart widgets
pub use chart::{BarChart, DataPoint, FinanceChart, LineChart, PieChart, Sparkline};
