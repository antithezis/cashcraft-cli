//! Terminal User Interface setup and event loop
//!
//! This module handles:
//! - Terminal initialization (raw mode, alternate screen)
//! - Main event loop with keyboard handling
//! - Terminal cleanup on exit
//! - Panic handler to restore terminal state

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

use crate::app::{App, Mode, StatusSeverity, View};
use crate::config::{self, Settings};
use crate::repository::Database;
use crate::ui::layout::main_layout;
use crate::ui::views::{
    BudgetState, BudgetView, ChartsState, ChartsView, Dashboard, DashboardState, ExpensesState,
    ExpensesView, IncomeState, IncomeView, PlaygroundState, PlaygroundView, SettingsState,
    SettingsView, TransactionsState, TransactionsView,
};
use crate::Result;

/// Type alias for our terminal type
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI mode
pub fn init() -> Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;

    // Install panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = restore();
        original_hook(panic);
    }));

    Ok(terminal)
}

/// Restore the terminal to normal mode
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// View-specific state container
pub struct ViewStates {
    pub dashboard: DashboardState,
    pub income: IncomeState,
    pub expenses: ExpensesState,
    pub transactions: TransactionsState,
    pub budget: BudgetState,
    pub charts: ChartsState,
    pub settings: SettingsState,
    pub playground: PlaygroundState,
}

impl ViewStates {
    /// Create new view states
    pub fn new(settings: &Settings) -> Self {
        let mut settings_state = SettingsState::new();
        settings_state.load(settings);

        Self {
            dashboard: DashboardState::new(),
            income: IncomeState::new(),
            expenses: ExpensesState::new(),
            transactions: TransactionsState::new(),
            budget: BudgetState::new(),
            charts: ChartsState::new(),
            settings: settings_state,
            playground: PlaygroundState::new(),
        }
    }

    /// Refresh all view data from database
    pub fn refresh_all(&mut self, db: &Database) {
        self.dashboard.refresh(db);
        self.income.refresh(db);
        self.expenses.refresh(db);
        self.transactions.refresh(db);
        self.budget.refresh(db);
        self.charts.refresh(db);
    }

    /// Refresh only the current view
    pub fn refresh_current(&mut self, view: View, db: &Database) {
        match view {
            View::Dashboard => self.dashboard.refresh(db),
            View::Income => self.income.refresh(db),
            View::Expenses => self.expenses.refresh(db),
            View::Transactions => self.transactions.refresh(db),
            View::Budget => self.budget.refresh(db),
            View::Charts => self.charts.refresh(db),
            View::Settings => {} // Settings don't need DB refresh
            View::Playground => self.playground.refresh(db),
        }
    }
}

/// Main TUI runner
pub struct TuiRunner {
    app: App,
    terminal: Tui,
    db: Database,
    view_states: ViewStates,
}

impl TuiRunner {
    /// Create a new TUI runner
    pub fn new(settings: Settings, keybindings: crate::config::Keybindings) -> Result<Self> {
        let terminal = init()?;
        let db_path = config::database_path();
        let db = Database::open(&db_path)?;
        let view_states = ViewStates::new(&settings);
        let app = App::new(settings, keybindings);

        Ok(Self {
            app,
            terminal,
            db,
            view_states,
        })
    }

    /// Run the main event loop
    pub fn run(&mut self) -> Result<()> {
        // Initial data load
        self.view_states.refresh_all(&self.db);

        while self.app.is_running() {
            // Draw the UI - pass references to avoid borrow conflicts
            let app = &self.app;
            let view_states = &self.view_states;
            self.terminal.draw(|frame| {
                draw_ui(frame, app, view_states);
            })?;

            // Handle events with 16ms timeout (~60 FPS)
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }

        Ok(())
    }

    /// Get the current view index for tab highlighting
    fn current_view_index(&self) -> usize {
        View::all()
            .iter()
            .position(|v| *v == self.app.view)
            .unwrap_or(0)
    }

    /// Handle a key event
    fn handle_key(&mut self, key: KeyEvent) {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.app.quit();
            return;
        }

        match self.app.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent) {
        // Handle "g" prefix combinations first
        if self.app.key_sequence() == "g" {
            match key.code {
                KeyCode::Char('g') => {
                    // "gg" - go to charts
                    self.app.set_view(View::Charts);
                    self.view_states.refresh_current(View::Charts, &self.db);
                    self.app.clear_key_buffer();
                    return;
                }
                KeyCode::Char('h') => {
                    self.app.set_view(View::Dashboard);
                    self.view_states.refresh_current(View::Dashboard, &self.db);
                    self.app.clear_key_buffer();
                    return;
                }
                KeyCode::Char('i') => {
                    self.app.set_view(View::Income);
                    self.view_states.refresh_current(View::Income, &self.db);
                    self.app.clear_key_buffer();
                    return;
                }
                KeyCode::Char('e') => {
                    self.app.set_view(View::Expenses);
                    self.view_states.refresh_current(View::Expenses, &self.db);
                    self.app.clear_key_buffer();
                    return;
                }
                KeyCode::Char('t') => {
                    self.app.set_view(View::Transactions);
                    self.view_states
                        .refresh_current(View::Transactions, &self.db);
                    self.app.clear_key_buffer();
                    return;
                }
                KeyCode::Char('b') => {
                    self.app.set_view(View::Budget);
                    self.view_states.refresh_current(View::Budget, &self.db);
                    self.app.clear_key_buffer();
                    return;
                }
                KeyCode::Char('p') => {
                    self.app.set_view(View::Playground);
                    self.view_states.refresh_current(View::Playground, &self.db);
                    self.app.clear_key_buffer();
                    return;
                }
                KeyCode::Char('s') => {
                    self.app.set_view(View::Settings);
                    self.app.clear_key_buffer();
                    return;
                }
                _ => {
                    // Unknown g-combination, clear buffer
                    self.app.clear_key_buffer();
                }
            }
        }

        // Standard key handling
        match key.code {
            // Quit
            KeyCode::Char('q') => {
                self.app.quit();
            }

            // Enter command mode
            KeyCode::Char(':') => {
                self.app.enter_command_mode();
            }

            // Enter insert mode
            KeyCode::Char('i') | KeyCode::Char('a') => {
                if self.app.view == View::Income && key.code == KeyCode::Char('a') {
                    self.view_states.income.form = crate::ui::views::income::IncomeFormState::default();
                    self.view_states.income.form.is_open = true;
                } else if self.app.view == View::Expenses && key.code == KeyCode::Char('a') {
                    self.view_states.expenses.form = crate::ui::views::expenses::ExpenseFormState::default();
                    self.view_states.expenses.form.is_open = true;
                } else if self.app.view == View::Transactions && key.code == KeyCode::Char('a') {
                    self.view_states.transactions.form = crate::ui::views::transactions::TransactionFormState::default();
                    self.view_states.transactions.form.is_open = true;
                } else if self.app.view == View::Budget && key.code == KeyCode::Char('a') {
                    self.view_states.budget.form = crate::ui::views::budget::BudgetFormState::default();
                    self.view_states.budget.form.is_open = true;
                }
                self.app.enter_insert_mode();
            }

            // Start navigation prefix 'g'
            KeyCode::Char('g') => {
                self.app.push_key('g');
            }

            // Tab cycles views
            KeyCode::Tab => {
                let views = View::all();
                let current = self.current_view_index();
                let next = (current + 1) % views.len();
                self.app.set_view(views[next]);
                self.view_states.refresh_current(views[next], &self.db);
            }

            // Shift+Tab cycles backward
            KeyCode::BackTab => {
                let views = View::all();
                let current = self.current_view_index();
                let prev = if current == 0 {
                    views.len() - 1
                } else {
                    current - 1
                };
                self.app.set_view(views[prev]);
                self.view_states.refresh_current(views[prev], &self.db);
            }

            // Escape clears key buffer
            KeyCode::Esc => {
                self.app.clear_key_buffer();
                self.app.clear_status();
            }

            // Refresh current view
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.view_states.refresh_current(self.app.view, &self.db);
                self.app.set_success("Refreshed");
            }

            // View-specific navigation (j/k)
            KeyCode::Char('j') | KeyCode::Down => {
                self.navigate_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.navigate_up();
            }

            // Number keys for quick view access (1-8)
            KeyCode::Char(c @ '1'..='8') => {
                let idx = c.to_digit(10).unwrap() as usize - 1;
                let views = View::all();
                if idx < views.len() {
                    self.app.set_view(views[idx]);
                    self.view_states.refresh_current(views[idx], &self.db);
                }
            }

            // Help
            KeyCode::Char('?') => {
                self.app
                    .set_info("Press q to quit, :help for commands, g<key> to navigate views");
            }

            // Back navigation
            KeyCode::Backspace => {
                self.app.go_back();
                self.view_states.refresh_current(self.app.view, &self.db);
            }

            _ => {}
        }
    }

    /// Handle keys in insert mode
    fn handle_insert_mode(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.app.enter_normal_mode();
            match self.app.view {
                View::Income => {
                    self.view_states.income.form.is_open = false;
                }
                View::Expenses => {
                    self.view_states.expenses.form.is_open = false;
                }
                View::Transactions => {
                    self.view_states.transactions.form.is_open = false;
                }
                View::Budget => {
                    self.view_states.budget.form.is_open = false;
                }
                _ => {}
            }
            return;
        }

        // Route insert event to active view's input state
        match self.app.view {
            View::Income => {
                if !self.view_states.income.form.is_open {
                    self.app.enter_normal_mode();
                    return;
                }
                match key.code {
                    KeyCode::Enter => {
                        // TODO: trigger save logic to db
                        use rust_decimal::Decimal;
                        let amount = self.view_states.income.form.amount.value().parse::<Decimal>().unwrap_or(Decimal::ZERO);
                        
                        let source = crate::domain::income::IncomeSource::new(
                            self.view_states.income.form.var_name.value().to_string(),
                            self.view_states.income.form.display_name.value().to_string(),
                            amount,
                            crate::ui::views::income::frequencies()[self.view_states.income.form.frequency_idx].clone()
                        );
                        
                        let service = crate::services::IncomeService::new(&self.db);
                        if let Err(e) = service.create(&source) {
                            self.view_states.income.form.error = Some(format!("Error: {}", e));
                        } else {
                            self.view_states.refresh_current(View::Income, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.income.form.is_open = false;
                            self.app.set_success("Income source added");
                        }
                    }
                    KeyCode::Tab => {
                        self.view_states.income.form.active_field = (self.view_states.income.form.active_field + 1) % 4;
                    }
                    KeyCode::BackTab => {
                        self.view_states.income.form.active_field = (self.view_states.income.form.active_field + 3) % 4;
                    }
                    KeyCode::Left => {
                        if self.view_states.income.form.active_field == 3 {
                            let idx = self.view_states.income.form.frequency_idx;
                            self.view_states.income.form.frequency_idx = if idx == 0 { 6 } else { idx - 1 };
                        } else {
                            match self.view_states.income.form.active_field {
                                0 => self.view_states.income.form.var_name.move_left(),
                                1 => self.view_states.income.form.display_name.move_left(),
                                2 => self.view_states.income.form.amount.move_left(),
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Right => {
                        if self.view_states.income.form.active_field == 3 {
                            self.view_states.income.form.frequency_idx = (self.view_states.income.form.frequency_idx + 1) % 7;
                        } else {
                            match self.view_states.income.form.active_field {
                                0 => self.view_states.income.form.var_name.move_right(),
                                1 => self.view_states.income.form.display_name.move_right(),
                                2 => self.view_states.income.form.amount.move_right(),
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Home => {
                        match self.view_states.income.form.active_field {
                            0 => self.view_states.income.form.var_name.move_start(),
                            1 => self.view_states.income.form.display_name.move_start(),
                            2 => self.view_states.income.form.amount.move_start(),
                            _ => {}
                        }
                    }
                    KeyCode::End => {
                        match self.view_states.income.form.active_field {
                            0 => self.view_states.income.form.var_name.move_end(),
                            1 => self.view_states.income.form.display_name.move_end(),
                            2 => self.view_states.income.form.amount.move_end(),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        match self.view_states.income.form.active_field {
                            0 => self.view_states.income.form.var_name.delete(),
                            1 => self.view_states.income.form.display_name.delete(),
                            2 => self.view_states.income.form.amount.delete(),
                            _ => {}
                        }
                    }
                    KeyCode::Delete => {
                        match self.view_states.income.form.active_field {
                            0 => self.view_states.income.form.var_name.delete_forward(),
                            1 => self.view_states.income.form.display_name.delete_forward(),
                            2 => self.view_states.income.form.amount.delete_forward(),
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => {
                        match self.view_states.income.form.active_field {
                            0 => self.view_states.income.form.var_name.insert(c),
                            1 => self.view_states.income.form.display_name.insert(c),
                            2 => self.view_states.income.form.amount.insert(c),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            View::Expenses => {
                if !self.view_states.expenses.form.is_open {
                    self.app.enter_normal_mode();
                    return;
                }
                match key.code {
                    KeyCode::Enter => {
                        use rust_decimal::Decimal;
                        let amount = self.view_states.expenses.form.amount.value().parse::<Decimal>().unwrap_or(Decimal::ZERO);
                        
                        let source = crate::domain::expense::Expense::new(
                            self.view_states.expenses.form.var_name.value().to_string(),
                            self.view_states.expenses.form.display_name.value().to_string(),
                            amount,
                            crate::ui::views::expenses::expense_types()[self.view_states.expenses.form.type_idx].clone(),
                            crate::ui::views::income::frequencies()[self.view_states.expenses.form.frequency_idx].clone(),
                            crate::ui::views::expenses::expense_categories()[self.view_states.expenses.form.category_idx].clone()
                        );
                        
                        let service = crate::services::ExpenseService::new(&self.db);
                        if let Err(e) = service.create(&source) {
                            self.view_states.expenses.form.error = Some(format!("Error: {}", e));
                        } else {
                            self.view_states.refresh_current(View::Expenses, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.expenses.form.is_open = false;
                            self.app.set_success("Expense added");
                        }
                    }
                    KeyCode::Tab => {
                        self.view_states.expenses.form.active_field = (self.view_states.expenses.form.active_field + 1) % 6;
                    }
                    KeyCode::BackTab => {
                        self.view_states.expenses.form.active_field = (self.view_states.expenses.form.active_field + 5) % 6;
                    }
                    KeyCode::Left => {
                        match self.view_states.expenses.form.active_field {
                            3 => {
                                let idx = self.view_states.expenses.form.type_idx;
                                self.view_states.expenses.form.type_idx = if idx == 0 { 2 } else { idx - 1 };
                            }
                            4 => {
                                let idx = self.view_states.expenses.form.frequency_idx;
                                self.view_states.expenses.form.frequency_idx = if idx == 0 { 6 } else { idx - 1 };
                            }
                            5 => {
                                let len = crate::ui::views::expenses::expense_categories().len();
                                let idx = self.view_states.expenses.form.category_idx;
                                self.view_states.expenses.form.category_idx = if idx == 0 { len - 1 } else { idx - 1 };
                            }
                            0 => self.view_states.expenses.form.var_name.move_left(),
                            1 => self.view_states.expenses.form.display_name.move_left(),
                            2 => self.view_states.expenses.form.amount.move_left(),
                            _ => {}
                        }
                    }
                    KeyCode::Right => {
                        match self.view_states.expenses.form.active_field {
                            3 => {
                                self.view_states.expenses.form.type_idx = (self.view_states.expenses.form.type_idx + 1) % 3;
                            }
                            4 => {
                                self.view_states.expenses.form.frequency_idx = (self.view_states.expenses.form.frequency_idx + 1) % 7;
                            }
                            5 => {
                                let len = crate::ui::views::expenses::expense_categories().len();
                                self.view_states.expenses.form.category_idx = (self.view_states.expenses.form.category_idx + 1) % len;
                            }
                            0 => self.view_states.expenses.form.var_name.move_right(),
                            1 => self.view_states.expenses.form.display_name.move_right(),
                            2 => self.view_states.expenses.form.amount.move_right(),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        match self.view_states.expenses.form.active_field {
                            0 => self.view_states.expenses.form.var_name.delete(),
                            1 => self.view_states.expenses.form.display_name.delete(),
                            2 => self.view_states.expenses.form.amount.delete(),
                            _ => {}
                        }
                    }
                    KeyCode::Delete => {
                        match self.view_states.expenses.form.active_field {
                            0 => self.view_states.expenses.form.var_name.delete_forward(),
                            1 => self.view_states.expenses.form.display_name.delete_forward(),
                            2 => self.view_states.expenses.form.amount.delete_forward(),
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => {
                        match self.view_states.expenses.form.active_field {
                            0 => self.view_states.expenses.form.var_name.insert(c),
                            1 => self.view_states.expenses.form.display_name.insert(c),
                            2 => self.view_states.expenses.form.amount.insert(c),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            View::Transactions => {
                if !self.view_states.transactions.form.is_open {
                    self.app.enter_normal_mode();
                    return;
                }
                match key.code {
                    KeyCode::Enter => {
                        use rust_decimal::Decimal;
                        let amount = self.view_states.transactions.form.amount.value().parse::<Decimal>().unwrap_or(Decimal::ZERO);
                        
                        let date_str = self.view_states.transactions.form.date.value();
                        let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or_else(|_| chrono::Local::now().date_naive());
                        
                        let source = crate::domain::transaction::Transaction::new(
                            date,
                            self.view_states.transactions.form.description.value().to_string(),
                            amount,
                            crate::ui::views::transactions::transaction_types()[self.view_states.transactions.form.type_idx].clone(),
                            self.view_states.transactions.form.category.value().to_string()
                        );
                        
                        let service = crate::services::TransactionService::new(&self.db);
                        if let Err(e) = service.create(&source) {
                            self.view_states.transactions.form.error = Some(format!("Error: {}", e));
                        } else {
                            self.view_states.refresh_current(View::Transactions, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.transactions.form.is_open = false;
                            self.app.set_success("Transaction added");
                        }
                    }
                    KeyCode::Tab => {
                        self.view_states.transactions.form.active_field = (self.view_states.transactions.form.active_field + 1) % 5;
                    }
                    KeyCode::BackTab => {
                        self.view_states.transactions.form.active_field = (self.view_states.transactions.form.active_field + 4) % 5;
                    }
                    KeyCode::Left => {
                        match self.view_states.transactions.form.active_field {
                            3 => {
                                let idx = self.view_states.transactions.form.type_idx;
                                self.view_states.transactions.form.type_idx = if idx == 0 { 2 } else { idx - 1 };
                            }
                            0 => self.view_states.transactions.form.date.move_left(),
                            1 => self.view_states.transactions.form.description.move_left(),
                            2 => self.view_states.transactions.form.amount.move_left(),
                            4 => self.view_states.transactions.form.category.move_left(),
                            _ => {}
                        }
                    }
                    KeyCode::Right => {
                        match self.view_states.transactions.form.active_field {
                            3 => {
                                self.view_states.transactions.form.type_idx = (self.view_states.transactions.form.type_idx + 1) % 3;
                            }
                            0 => self.view_states.transactions.form.date.move_right(),
                            1 => self.view_states.transactions.form.description.move_right(),
                            2 => self.view_states.transactions.form.amount.move_right(),
                            4 => self.view_states.transactions.form.category.move_right(),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        match self.view_states.transactions.form.active_field {
                            0 => self.view_states.transactions.form.date.delete(),
                            1 => self.view_states.transactions.form.description.delete(),
                            2 => self.view_states.transactions.form.amount.delete(),
                            4 => self.view_states.transactions.form.category.delete(),
                            _ => {}
                        }
                    }
                    KeyCode::Delete => {
                        match self.view_states.transactions.form.active_field {
                            0 => self.view_states.transactions.form.date.delete_forward(),
                            1 => self.view_states.transactions.form.description.delete_forward(),
                            2 => self.view_states.transactions.form.amount.delete_forward(),
                            4 => self.view_states.transactions.form.category.delete_forward(),
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => {
                        match self.view_states.transactions.form.active_field {
                            0 => self.view_states.transactions.form.date.insert(c),
                            1 => self.view_states.transactions.form.description.insert(c),
                            2 => self.view_states.transactions.form.amount.insert(c),
                            4 => self.view_states.transactions.form.category.insert(c),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            View::Budget => {
                if !self.view_states.budget.form.is_open {
                    self.app.enter_normal_mode();
                    return;
                }
                match key.code {
                    KeyCode::Enter => {
                        use rust_decimal::Decimal;
                        let amount = self.view_states.budget.form.amount.value().parse::<Decimal>().unwrap_or(Decimal::ZERO);
                        let category = self.view_states.budget.form.category.value().to_string();
                        
                        let month = self.view_states.budget.month;
                        let year = self.view_states.budget.year;
                        
                        let source = crate::domain::budget::Budget::new(
                            month,
                            year,
                            category,
                            amount
                        );
                        
                        let service = crate::services::BudgetService::new(&self.db);
                        if let Err(e) = service.create(&source) {
                            self.view_states.budget.form.error = Some(format!("Error: {}", e));
                        } else {
                            self.view_states.refresh_current(View::Budget, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.budget.form.is_open = false;
                            self.app.set_success("Budget added");
                        }
                    }
                    KeyCode::Tab => {
                        self.view_states.budget.form.active_field = (self.view_states.budget.form.active_field + 1) % 2;
                    }
                    KeyCode::BackTab => {
                        self.view_states.budget.form.active_field = (self.view_states.budget.form.active_field + 1) % 2;
                    }
                    KeyCode::Left => {
                        match self.view_states.budget.form.active_field {
                            0 => self.view_states.budget.form.category.move_left(),
                            1 => self.view_states.budget.form.amount.move_left(),
                            _ => {}
                        }
                    }
                    KeyCode::Right => {
                        match self.view_states.budget.form.active_field {
                            0 => self.view_states.budget.form.category.move_right(),
                            1 => self.view_states.budget.form.amount.move_right(),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace => {
                        match self.view_states.budget.form.active_field {
                            0 => self.view_states.budget.form.category.delete(),
                            1 => self.view_states.budget.form.amount.delete(),
                            _ => {}
                        }
                    }
                    KeyCode::Delete => {
                        match self.view_states.budget.form.active_field {
                            0 => self.view_states.budget.form.category.delete_forward(),
                            1 => self.view_states.budget.form.amount.delete_forward(),
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => {
                        match self.view_states.budget.form.active_field {
                            0 => self.view_states.budget.form.category.insert(c),
                            1 => self.view_states.budget.form.amount.insert(c),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            View::Playground => {
                match key.code {
                    KeyCode::Enter => {
                        self.view_states.playground.evaluate();
                    }
                    KeyCode::Char(c) => {
                        self.view_states.playground.input.insert(c);
                    }
                    KeyCode::Backspace => {
                        self.view_states.playground.input.delete();
                    }
                    KeyCode::Delete => {
                        self.view_states.playground.input.delete_forward();
                    }
                    KeyCode::Left => {
                        self.view_states.playground.input.move_left();
                    }
                    KeyCode::Right => {
                        self.view_states.playground.input.move_right();
                    }
                    KeyCode::Home => {
                        self.view_states.playground.input.move_start();
                    }
                    KeyCode::End => {
                        self.view_states.playground.input.move_end();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Handle keys in command mode
    fn handle_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.app.enter_normal_mode();
            }
            KeyCode::Enter => {
                self.execute_command();
                self.app.enter_normal_mode();
            }
            KeyCode::Backspace => {
                self.app.pop_command_char();
                if self.app.command().is_empty() {
                    self.app.enter_normal_mode();
                }
            }
            KeyCode::Char(c) => {
                self.app.push_command_char(c);
            }
            _ => {}
        }
    }

    /// Execute the current command
    fn execute_command(&mut self) {
        let command = self.app.command().trim().to_lowercase();

        match command.as_str() {
            "q" | "quit" => {
                self.app.quit();
            }
            "w" | "write" | "save" => {
                // Save settings
                if let Err(e) = self.app.settings.save(&Settings::default_path()) {
                    self.app.set_error(format!("Failed to save: {}", e));
                } else {
                    self.app.set_success("Settings saved");
                }
            }
            "wq" => {
                if let Err(e) = self.app.settings.save(&Settings::default_path()) {
                    self.app.set_error(format!("Failed to save: {}", e));
                } else {
                    self.app.quit();
                }
            }
            "refresh" | "r" => {
                self.view_states.refresh_all(&self.db);
                self.app.set_success("All views refreshed");
            }
            "help" | "h" => {
                self.app.set_info(
                    "Commands: :q(uit), :w(rite), :wq, :refresh, :theme <name>, :export <format>",
                );
            }
            cmd if cmd.starts_with("theme ") => {
                let theme_name = cmd.strip_prefix("theme ").unwrap().trim();
                self.app.set_theme(theme_name);
            }
            cmd if cmd.starts_with("export ") => {
                let format = cmd.strip_prefix("export ").unwrap().trim();
                self.app
                    .set_info(format!("Export to {} - coming soon!", format));
            }
            _ => {
                self.app.set_error(format!("Unknown command: {}", command));
            }
        }
    }

    /// Navigate down in the current view
    fn navigate_down(&mut self) {
        match self.app.view {
            View::Income => self.view_states.income.next(),
            View::Expenses => self.view_states.expenses.next(),
            View::Transactions => self.view_states.transactions.next(),
            View::Budget => self.view_states.budget.next(),
            View::Settings => self.view_states.settings.next(),
            _ => {}
        }
    }

    /// Navigate up in the current view
    fn navigate_up(&mut self) {
        match self.app.view {
            View::Income => self.view_states.income.previous(),
            View::Expenses => self.view_states.expenses.previous(),
            View::Transactions => self.view_states.transactions.previous(),
            View::Budget => self.view_states.budget.previous(),
            View::Settings => self.view_states.settings.previous(),
            _ => {}
        }
    }
}

impl Drop for TuiRunner {
    fn drop(&mut self) {
        let _ = restore();
    }
}

// ============================================================================
// Drawing functions (free functions to avoid borrow conflicts)
// ============================================================================

/// Draw the entire UI
fn draw_ui(frame: &mut Frame, app: &App, view_states: &ViewStates) {
    let area = frame.area();

    // Main layout: header, content, footer
    let (header_area, content_area, footer_area) = main_layout(area);

    // Draw header with tabs
    draw_header(frame, app, header_area);

    // Draw the current view
    draw_view(frame, app, view_states, content_area);

    // Draw footer with status and help
    draw_footer(frame, app, footer_area);
}

/// Draw the header with view tabs
fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // Create tab titles
    let titles: Vec<Line> = View::all()
        .iter()
        .map(|v| {
            let style = if *v == app.view {
                Style::default()
                    .fg(theme.colors.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.colors.text_muted)
            };
            Line::from(Span::styled(v.name(), style))
        })
        .collect();

    let current_index = View::all().iter().position(|v| *v == app.view).unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.colors.border))
                .title(Span::styled(
                    " CashCraft ",
                    Style::default()
                        .fg(theme.colors.accent)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .select(current_index)
        .highlight_style(
            Style::default()
                .fg(theme.colors.accent)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" | "));

    frame.render_widget(tabs, area);
}

/// Draw the current view content
fn draw_view(frame: &mut Frame, app: &App, view_states: &ViewStates, area: Rect) {
    let theme = &app.theme;

    match app.view {
        View::Dashboard => {
            let dashboard = Dashboard::new(&view_states.dashboard, theme);
            frame.render_widget(dashboard, area);
        }
        View::Income => {
            let income = IncomeView::new(&view_states.income, theme);
            frame.render_widget(income, area);
        }
        View::Expenses => {
            let expenses = ExpensesView::new(&view_states.expenses, theme);
            frame.render_widget(expenses, area);
        }
        View::Transactions => {
            let transactions = TransactionsView::new(&view_states.transactions, theme);
            frame.render_widget(transactions, area);
        }
        View::Budget => {
            let budget = BudgetView::new(&view_states.budget, theme);
            frame.render_widget(budget, area);
        }
        View::Charts => {
            let charts = ChartsView::new(&view_states.charts, theme);
            frame.render_widget(charts, area);
        }
        View::Settings => {
            let settings_view = SettingsView::new(&view_states.settings, theme);
            frame.render_widget(settings_view, area);
        }
        View::Playground => {
            let playground = PlaygroundView::new(&view_states.playground, theme);
            frame.render_widget(playground, area);
        }
    }
}

/// Draw the footer with status and help
fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // Build footer text
    let mode_style = match app.mode {
        Mode::Normal => Style::default()
            .fg(theme.colors.accent)
            .add_modifier(Modifier::BOLD),
        Mode::Insert => Style::default()
            .fg(theme.colors.success)
            .add_modifier(Modifier::BOLD),
        Mode::Command => Style::default()
            .fg(theme.colors.warning)
            .add_modifier(Modifier::BOLD),
    };

    let mut spans = vec![
        Span::styled(app.mode.indicator(), mode_style),
        Span::raw(" "),
    ];

    // Show command buffer in command mode
    if app.mode == Mode::Command {
        spans.push(Span::styled(
            format!(":{}", app.command()),
            Style::default().fg(theme.colors.text_primary),
        ));
    } else if let Some(msg) = &app.status_message {
        // Show status message
        let status_style = match msg.severity {
            StatusSeverity::Info => Style::default().fg(theme.colors.text_secondary),
            StatusSeverity::Success => Style::default().fg(theme.colors.success),
            StatusSeverity::Warning => Style::default().fg(theme.colors.warning),
            StatusSeverity::Error => Style::default().fg(theme.colors.error),
        };
        spans.push(Span::styled(&msg.text, status_style));
    } else {
        // Show help hints
        let help = match app.view {
            View::Dashboard => "q:quit  g:goto  ?:help",
            View::Income | View::Expenses => "j/k:nav  a:add  e:edit  d:delete  q:quit",
            View::Transactions => "j/k:nav  a:add  /:search  q:quit",
            View::Budget => "j/k:nav  a:add  Tab:month  q:quit",
            View::Playground => "Enter:eval  Ctrl-C:clear  q:quit",
            View::Charts => "h/l:period  Tab:chart  q:quit",
            View::Settings => "j/k:nav  Enter:change  q:quit",
        };
        spans.push(Span::styled(
            help,
            Style::default().fg(theme.colors.text_muted),
        ));
    }

    // Show key buffer if not empty
    if !app.key_buffer.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("[{}]", app.key_sequence()),
            Style::default().fg(theme.colors.text_muted),
        ));
    }

    let footer = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.colors.border)),
    );

    frame.render_widget(footer, area);
}
