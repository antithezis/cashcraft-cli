//! User interface layer for CashCraft TUI
//!
//! Ratatui-based TUI with Vim-style navigation, featuring:
//! - Theme system with 10 built-in themes
//! - Layout helpers for consistent UI structure
//! - Animation system for smooth transitions
//! - Custom widgets (tables, inputs, charts, progress bars)
//! - Views for each feature area

pub mod animation;
pub mod layout;
pub mod theme;
pub mod tui;
pub mod views;
pub mod widgets;

// Theme exports
pub use theme::{Theme, ThemeColors, ThemeVariant};

// Layout exports
pub use layout::{
    centered, centered_horizontal, centered_vertical, columns, form_layout, is_usable, main_layout,
    modal, rows, scroll_range, sidebar_layout, split_horizontal, split_three_horizontal,
    split_vertical, with_margin, with_margin_asymmetric, with_margin_individual, ConstraintBuilder,
    BORDER_WIDTH, MIN_CONTENT_HEIGHT, MIN_CONTENT_WIDTH, PADDING,
};

// Animation exports
pub use animation::{
    Animation, AnimationController, AnimationSpeed, Easing, FrameTimer, NumberCounter, Pulse,
    Transition,
};

// Re-export widgets module for convenience
pub use widgets::{
    // Table
    Alignment,
    // Charts
    BarChart,
    // Progress
    BudgetProgress,
    CircularProgress,
    ColumnWidth,
    DataPoint,
    FinanceChart,
    // Input
    InputState,
    LineChart,
    MiniProgress,
    MultiLineInputState,
    PieChart,
    ProgressBar,
    ProgressStyle,
    SimpleTable,
    Sparkline,
    TableColumn,
    TableRow,
    TableState,
    TextInput,
    VimTable,
};
