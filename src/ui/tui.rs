//! Terminal User Interface setup and event loop
//!
//! This module handles:
//! - Terminal initialization (raw mode, alternate screen)
//! - Main event loop with keyboard handling
//! - Terminal cleanup on exit
//! - Panic handler to restore terminal state

#[allow(unused_imports)]
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
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

use crate::app::{App, Mode, PendingAction, StatusSeverity, View};
use crate::config::{self, Settings};
use crate::repository::Database;
use crate::ui::history::Action;
use crate::ui::layout::main_layout;
use crate::ui::views::{
    BudgetState, BudgetView, ChartsState, ChartsView, Dashboard, DashboardState, ExpensesState,
    ExpensesView, IncomeState, IncomeView, PlaygroundState, PlaygroundView, SettingsSection,
    SettingsState, SettingsView, TransactionsState, TransactionsView,
};
use crate::Result;

/// Type alias for our terminal type
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI mode
pub fn init() -> Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
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
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
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
                match event::read()? {
                    Event::Key(key) => self.handle_key(key),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    _ => {}
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

        // Handle confirmation modal input
        if self.app.show_confirmation {
            self.handle_confirmation(key);
            return;
        }

        match self.app.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
        }
    }

    /// Handle mouse events
    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if let MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse.kind {
            let (col, row) = (mouse.column, mouse.row);
            // Get terminal size. Using unwrap_or_default just in case.
            // If it returns Size, we convert to Rect. If Rect, we use it.
            // But main_layout needs Rect.
            let size = self.terminal.size().unwrap_or_default();
            let area = Rect::new(0, 0, size.width, size.height);

            let (header_area, _content_area, _) = crate::ui::layout::main_layout(area);

            // Handle Tab clicks
            if row >= header_area.y && row < header_area.y + header_area.height {
                let views = View::all();

                // Calculate tab positions based on how Tabs widget renders:
                // The tabs are rendered inside a Block with borders, after the title.
                // Block has 1 char border on left. Title " CashCraft " is 12 chars.
                // Tabs start after the title with some padding.
                // Ratatui's Tabs widget renders: "tab1 | tab2 | tab3"
                // Each tab width = label.len(), separator = " | " (3 chars)
                //
                // Actually, looking at how ratatui renders Tabs:
                // - Block left border: 1 char
                // - Title: " CashCraft " (12 chars)
                // - Then tabs start immediately after
                //
                // However the actual tab content starts at x=1 (after left border)
                // and includes the title which is part of the block decoration.
                // The tabs themselves start after the left border, no matter the title.
                let left_border = 1u16;
                let mut x_offset = header_area.x + left_border;

                for &view in views {
                    let title = view.name();
                    let width = title.len() as u16;
                    let separator_width = 3u16; // " | "

                    // Check if click is within this tab's area
                    if col >= x_offset && col < x_offset + width {
                        if self.app.view != view {
                            self.app.set_view(view);
                            self.view_states.refresh_current(view, &self.db);
                        }
                        return;
                    }

                    x_offset += width + separator_width;
                }
            }
        }
    }

    /// Handle confirmation modal input
    fn handle_confirmation(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.execute_pending_action();
                self.app.show_confirmation = false;
                self.app.pending_action = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.app.show_confirmation = false;
                self.app.pending_action = None;
                self.app.set_info("Action cancelled");
            }
            _ => {}
        }
    }

    /// Execute the pending action
    fn execute_pending_action(&mut self) {
        if let Some(action) = self.app.pending_action.clone() {
            match action {
                PendingAction::DeleteIncome(id) => {
                    let service = crate::services::IncomeService::new(&self.db);
                    if let Ok(Some(item)) = service.get_by_id(&id) {
                        if let Err(e) = service.delete(&id) {
                            self.app.set_error(format!("Error deleting: {}", e));
                        } else {
                            self.app.history.push(Action::DeleteIncome(item));
                            self.view_states.refresh_current(View::Income, &self.db);
                            self.app.set_success("Income source deleted (Undo: u)");
                        }
                    }
                }
                PendingAction::DeleteExpense(id) => {
                    let service = crate::services::ExpenseService::new(&self.db);
                    if let Ok(Some(item)) = service.get_by_id(&id) {
                        if let Err(e) = service.delete(&id) {
                            self.app.set_error(format!("Error deleting: {}", e));
                        } else {
                            self.app.history.push(Action::DeleteExpense(item));
                            self.view_states.refresh_current(View::Expenses, &self.db);
                            self.app.set_success("Expense deleted (Undo: u)");
                        }
                    }
                }
                PendingAction::DeleteTransaction(id) => {
                    let service = crate::services::TransactionService::new(&self.db);
                    if let Ok(Some(item)) = service.get_by_id(&id) {
                        if let Err(e) = service.delete(&id) {
                            self.app.set_error(format!("Error deleting: {}", e));
                        } else {
                            self.app.history.push(Action::DeleteTransaction(item));
                            self.view_states
                                .refresh_current(View::Transactions, &self.db);
                            self.app.set_success("Transaction deleted (Undo: u)");
                        }
                    }
                }
                PendingAction::DeleteBudget(id) => {
                    let service = crate::services::BudgetService::new(&self.db);
                    if let Ok(Some(item)) = service.get_by_id(&id) {
                        if let Err(e) = service.delete(&id) {
                            self.app.set_error(format!("Error deleting: {}", e));
                        } else {
                            self.app.history.push(Action::DeleteBudget(item));
                            self.view_states.refresh_current(View::Budget, &self.db);
                            self.app.set_success("Budget deleted (Undo: u)");
                        }
                    }
                }
            }
            self.app.pending_action = None;
        }
    }

    /// Undo last action
    fn undo_action(&mut self) {
        if let Some(action) = self.app.history.pop_undo() {
            match action.clone() {
                Action::DeleteIncome(item) => {
                    let service = crate::services::IncomeService::new(&self.db);
                    if let Err(e) = service.create(&item) {
                        self.app.set_error(format!("Undo failed: {}", e));
                        self.app.history.push_undo_only(action); // Put it back
                    } else {
                        self.app.history.push_redo(action);
                        self.view_states.refresh_current(View::Income, &self.db);
                        self.app.set_success("Undo: Income restored");
                    }
                }
                Action::DeleteExpense(item) => {
                    let service = crate::services::ExpenseService::new(&self.db);
                    if let Err(e) = service.create(&item) {
                        self.app.set_error(format!("Undo failed: {}", e));
                        self.app.history.push_undo_only(action);
                    } else {
                        self.app.history.push_redo(action);
                        self.view_states.refresh_current(View::Expenses, &self.db);
                        self.app.set_success("Undo: Expense restored");
                    }
                }
                Action::DeleteTransaction(item) => {
                    let service = crate::services::TransactionService::new(&self.db);
                    if let Err(e) = service.create(&item) {
                        self.app.set_error(format!("Undo failed: {}", e));
                        self.app.history.push_undo_only(action);
                    } else {
                        self.app.history.push_redo(action);
                        self.view_states
                            .refresh_current(View::Transactions, &self.db);
                        self.app.set_success("Undo: Transaction restored");
                    }
                }
                Action::DeleteBudget(item) => {
                    let service = crate::services::BudgetService::new(&self.db);
                    if let Err(e) = service.create(&item) {
                        self.app.set_error(format!("Undo failed: {}", e));
                        self.app.history.push_undo_only(action);
                    } else {
                        self.app.history.push_redo(action);
                        self.view_states.refresh_current(View::Budget, &self.db);
                        self.app.set_success("Undo: Budget restored");
                    }
                }
            }
        } else {
            self.app.set_info("Nothing to undo");
        }
    }

    /// Redo last undone action
    fn redo_action(&mut self) {
        if let Some(action) = self.app.history.pop_redo() {
            match action.clone() {
                Action::DeleteIncome(item) => {
                    let service = crate::services::IncomeService::new(&self.db);
                    if let Err(e) = service.delete(&item.id.to_string()) {
                        self.app.set_error(format!("Redo failed: {}", e));
                        self.app.history.push_redo(action); // Put it back
                    } else {
                        self.app.history.push_undo_only(action);
                        self.view_states.refresh_current(View::Income, &self.db);
                        self.app.set_success("Redo: Income deleted");
                    }
                }
                Action::DeleteExpense(item) => {
                    let service = crate::services::ExpenseService::new(&self.db);
                    if let Err(e) = service.delete(&item.id.to_string()) {
                        self.app.set_error(format!("Redo failed: {}", e));
                        self.app.history.push_redo(action);
                    } else {
                        self.app.history.push_undo_only(action);
                        self.view_states.refresh_current(View::Expenses, &self.db);
                        self.app.set_success("Redo: Expense deleted");
                    }
                }
                Action::DeleteTransaction(item) => {
                    let service = crate::services::TransactionService::new(&self.db);
                    if let Err(e) = service.delete(&item.id.to_string()) {
                        self.app.set_error(format!("Redo failed: {}", e));
                        self.app.history.push_redo(action);
                    } else {
                        self.app.history.push_undo_only(action);
                        self.view_states
                            .refresh_current(View::Transactions, &self.db);
                        self.app.set_success("Redo: Transaction deleted");
                    }
                }
                Action::DeleteBudget(item) => {
                    let service = crate::services::BudgetService::new(&self.db);
                    if let Err(e) = service.delete(&item.id.to_string()) {
                        self.app.set_error(format!("Redo failed: {}", e));
                        self.app.history.push_redo(action);
                    } else {
                        self.app.history.push_undo_only(action);
                        self.view_states.refresh_current(View::Budget, &self.db);
                        self.app.set_success("Redo: Budget deleted");
                    }
                }
            }
        } else {
            self.app.set_info("Nothing to redo");
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
                    self.view_states.income.form =
                        crate::ui::views::income::IncomeFormState::default();
                    self.view_states.income.form.is_open = true;
                } else if self.app.view == View::Expenses && key.code == KeyCode::Char('a') {
                    self.view_states.expenses.form =
                        crate::ui::views::expenses::ExpenseFormState::default();
                    self.view_states.expenses.form.is_open = true;
                } else if self.app.view == View::Transactions && key.code == KeyCode::Char('a') {
                    self.view_states.transactions.form =
                        crate::ui::views::transactions::TransactionFormState::default();
                    self.view_states.transactions.form.is_open = true;
                } else if self.app.view == View::Budget && key.code == KeyCode::Char('a') {
                    self.view_states.budget.form =
                        crate::ui::views::budget::BudgetFormState::default();
                    self.view_states.budget.form.is_open = true;
                }
                self.app.enter_insert_mode();
            }

            // Start navigation prefix 'g'
            KeyCode::Char('g') => {
                self.app.push_key('g');
            }

            // Edit item
            KeyCode::Char('e') => {
                match self.app.view {
                    View::Income => {
                        if let Some(selected) = self.view_states.income.selected() {
                            let mut form = crate::ui::views::income::IncomeFormState::default();
                            form.is_open = true;
                            form.is_edit = true;
                            form.edit_id = Some(selected.id.to_string());
                            form.var_name.set_value(selected.variable_name.clone());
                            form.display_name.set_value(selected.display_name.clone());
                            form.amount.set_value(selected.amount.to_string());
                            if let Some(idx) = crate::ui::views::income::frequencies()
                                .iter()
                                .position(|f| *f == selected.frequency)
                            {
                                form.frequency_idx = idx;
                            }
                            self.view_states.income.form = form;
                            self.app.enter_insert_mode();
                        }
                    }
                    View::Expenses => {
                        if let Some(selected) = self.view_states.expenses.selected() {
                            let mut form = crate::ui::views::expenses::ExpenseFormState::default();
                            form.is_open = true;
                            form.is_edit = true;
                            form.edit_id = Some(selected.id.to_string());
                            form.var_name.set_value(selected.variable_name.clone());
                            form.display_name.set_value(selected.display_name.clone());
                            form.amount.set_value(selected.amount.to_string());

                            if let Some(idx) = crate::ui::views::expenses::expense_types()
                                .iter()
                                .position(|t| *t == selected.expense_type)
                            {
                                form.type_idx = idx;
                            }

                            if let Some(idx) = crate::ui::views::income::frequencies()
                                .iter()
                                .position(|f| *f == selected.frequency)
                            {
                                form.frequency_idx = idx;
                            }

                            if let Some(idx) = crate::ui::views::expenses::expense_categories()
                                .iter()
                                .position(|c| *c == selected.category)
                            {
                                form.category_idx = idx;
                            }

                            self.view_states.expenses.form = form;
                            self.app.enter_insert_mode();
                        }
                    }
                    View::Transactions => {
                        if let Some(selected) = self.view_states.transactions.selected_transaction()
                        {
                            let mut form =
                                crate::ui::views::transactions::TransactionFormState::default();
                            form.is_open = true;
                            form.is_edit = true;
                            form.edit_id = Some(selected.id.to_string());
                            form.date
                                .set_value(selected.date.format("%Y-%m-%d").to_string());
                            form.description.set_value(selected.description.clone());
                            form.amount.set_value(selected.amount.abs().to_string()); // Use abs value
                            form.category.set_value(selected.category.clone());

                            if let Some(idx) = crate::ui::views::transactions::transaction_types()
                                .iter()
                                .position(|t| *t == selected.transaction_type)
                            {
                                form.type_idx = idx;
                            }

                            self.view_states.transactions.form = form;
                            self.app.enter_insert_mode();
                        }
                    }
                    View::Budget => {
                        if let Some(selected) = self.view_states.budget.selected_budget() {
                            let mut form = crate::ui::views::budget::BudgetFormState::default();
                            form.is_open = true;
                            form.is_edit = true;
                            form.edit_id = Some(selected.id.to_string());
                            form.category.set_value(selected.category.clone());
                            form.amount.set_value(selected.amount.to_string());
                            form.is_template_mode = selected.is_template;

                            self.view_states.budget.form = form;
                            self.app.enter_insert_mode();
                        }
                    }
                    _ => {}
                }
            }

            // Delete item
            KeyCode::Char('d') => match self.app.view {
                View::Income => {
                    if let Some(selected) = self.view_states.income.selected() {
                        self.app.pending_action =
                            Some(PendingAction::DeleteIncome(selected.id.to_string()));
                        self.app.show_confirmation = true;
                    }
                }
                View::Expenses => {
                    if let Some(selected) = self.view_states.expenses.selected() {
                        self.app.pending_action =
                            Some(PendingAction::DeleteExpense(selected.id.to_string()));
                        self.app.show_confirmation = true;
                    }
                }
                View::Transactions => {
                    if let Some(selected) = self.view_states.transactions.selected_transaction() {
                        self.app.pending_action =
                            Some(PendingAction::DeleteTransaction(selected.id.to_string()));
                        self.app.show_confirmation = true;
                    }
                }
                View::Budget => {
                    if let Some(selected) = self.view_states.budget.selected_budget() {
                        self.app.pending_action =
                            Some(PendingAction::DeleteBudget(selected.id.to_string()));
                        self.app.show_confirmation = true;
                    }
                }
                _ => {}
            },

            // Undo
            KeyCode::Char('u') => {
                self.undo_action();
            }

            // Redo (overrides refresh)
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.redo_action();
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

            // Month navigation in Transactions and Budget
            KeyCode::Char('[') => {
                if self.app.view == View::Transactions {
                    self.view_states.transactions.prev_month();
                    self.view_states
                        .refresh_current(View::Transactions, &self.db);
                } else if self.app.view == View::Budget {
                    self.view_states.budget.prev_month();
                    self.view_states.refresh_current(View::Budget, &self.db);
                }
            }
            KeyCode::Char(']') => {
                if self.app.view == View::Transactions {
                    self.view_states.transactions.next_month();
                    self.view_states
                        .refresh_current(View::Transactions, &self.db);
                } else if self.app.view == View::Budget {
                    self.view_states.budget.next_month();
                    self.view_states.refresh_current(View::Budget, &self.db);
                }
            }

            // Create override for selected budget template (this month only)
            KeyCode::Char('o') => {
                if self.app.view == View::Budget {
                    if let Some(selected) = self.view_states.budget.selected_budget() {
                        // Create an override form pre-filled with template values
                        let mut form = crate::ui::views::budget::BudgetFormState::default();
                        form.is_open = true;
                        form.is_edit = false;
                        form.is_template_mode = false; // Creating an override, not a template
                        form.category.set_value(selected.category.clone());
                        form.amount.set_value(selected.amount.to_string());
                        self.view_states.budget.form = form;
                        self.app.enter_insert_mode();
                    }
                }
            }

            // Escape clears key buffer
            KeyCode::Esc => {
                self.app.clear_key_buffer();
                self.app.clear_status();
            }

            // Refresh current view (Shift+R)
            KeyCode::Char('R') => {
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

            // Enter key
            KeyCode::Enter => {
                if self.app.view == View::Settings {
                    self.view_states.settings.enter();
                    self.app.settings.appearance.animations_enabled = self
                        .view_states
                        .settings
                        .settings
                        .appearance
                        .animations_enabled;
                }
            }

            // Left/h navigation
            KeyCode::Char('h') | KeyCode::Left => {
                if self.app.view == View::Charts {
                    self.view_states.charts.prev_chart();
                } else if self.app.view == View::Settings {
                    if self.view_states.settings.section == SettingsSection::Appearance
                        && self.view_states.settings.table_state.selected == 0
                    {
                        self.view_states.settings.prev_value();
                        let new_theme = self.view_states.settings.settings.appearance.theme.clone();
                        self.app.set_theme(&new_theme);
                    } else {
                        self.view_states.settings.prev_section();
                    }
                }
            }

            // Right/l navigation
            KeyCode::Char('l') | KeyCode::Right => {
                if self.app.view == View::Charts {
                    self.view_states.charts.next_chart();
                } else if self.app.view == View::Settings {
                    if self.view_states.settings.section == SettingsSection::Appearance
                        && self.view_states.settings.table_state.selected == 0
                    {
                        self.view_states.settings.next_value();
                        let new_theme = self.view_states.settings.settings.appearance.theme.clone();
                        self.app.set_theme(&new_theme);
                    } else {
                        self.view_states.settings.next_section();
                    }
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
                        // Trigger save logic to db
                        use rust_decimal::Decimal;
                        let amount = self
                            .view_states
                            .income
                            .form
                            .amount
                            .value()
                            .parse::<Decimal>()
                            .unwrap_or(Decimal::ZERO);

                        let service = crate::services::IncomeService::new(&self.db);
                        let result = if self.view_states.income.form.is_edit {
                            if let Some(id_str) = &self.view_states.income.form.edit_id {
                                if let Ok(Some(mut source)) = service.get_by_id(id_str) {
                                    source.variable_name =
                                        self.view_states.income.form.var_name.value().to_string();
                                    source.display_name = self
                                        .view_states
                                        .income
                                        .form
                                        .display_name
                                        .value()
                                        .to_string();
                                    source.amount = amount;
                                    source.frequency = crate::ui::views::income::frequencies()
                                        [self.view_states.income.form.frequency_idx]
                                        .clone();

                                    service.update(&source)
                                } else {
                                    Err(crate::error::CashCraftError::Validation(
                                        "Record not found".to_string(),
                                    ))
                                }
                            } else {
                                Err(crate::error::CashCraftError::Validation(
                                    "Missing edit ID".to_string(),
                                ))
                            }
                        } else {
                            let source = crate::domain::income::IncomeSource::new(
                                self.view_states.income.form.var_name.value().to_string(),
                                self.view_states
                                    .income
                                    .form
                                    .display_name
                                    .value()
                                    .to_string(),
                                amount,
                                crate::ui::views::income::frequencies()
                                    [self.view_states.income.form.frequency_idx]
                                    .clone(),
                            );
                            service.create(&source)
                        };

                        if let Err(e) = result {
                            self.view_states.income.form.error = Some(format!("Error: {}", e));
                        } else {
                            self.view_states.refresh_current(View::Income, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.income.form.is_open = false;
                            let msg = if self.view_states.income.form.is_edit {
                                "Income source updated"
                            } else {
                                "Income source added"
                            };
                            self.app.set_success(msg);
                        }
                    }
                    KeyCode::Tab => {
                        self.view_states.income.form.active_field =
                            (self.view_states.income.form.active_field + 1) % 4;
                    }
                    KeyCode::BackTab => {
                        self.view_states.income.form.active_field =
                            (self.view_states.income.form.active_field + 3) % 4;
                    }
                    KeyCode::Left => {
                        if self.view_states.income.form.active_field == 3 {
                            let idx = self.view_states.income.form.frequency_idx;
                            self.view_states.income.form.frequency_idx =
                                if idx == 0 { 6 } else { idx - 1 };
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
                            self.view_states.income.form.frequency_idx =
                                (self.view_states.income.form.frequency_idx + 1) % 7;
                        } else {
                            match self.view_states.income.form.active_field {
                                0 => self.view_states.income.form.var_name.move_right(),
                                1 => self.view_states.income.form.display_name.move_right(),
                                2 => self.view_states.income.form.amount.move_right(),
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Home => match self.view_states.income.form.active_field {
                        0 => self.view_states.income.form.var_name.move_start(),
                        1 => self.view_states.income.form.display_name.move_start(),
                        2 => self.view_states.income.form.amount.move_start(),
                        _ => {}
                    },
                    KeyCode::End => match self.view_states.income.form.active_field {
                        0 => self.view_states.income.form.var_name.move_end(),
                        1 => self.view_states.income.form.display_name.move_end(),
                        2 => self.view_states.income.form.amount.move_end(),
                        _ => {}
                    },
                    KeyCode::Backspace => match self.view_states.income.form.active_field {
                        0 => self.view_states.income.form.var_name.delete(),
                        1 => self.view_states.income.form.display_name.delete(),
                        2 => self.view_states.income.form.amount.delete(),
                        _ => {}
                    },
                    KeyCode::Delete => match self.view_states.income.form.active_field {
                        0 => self.view_states.income.form.var_name.delete_forward(),
                        1 => self.view_states.income.form.display_name.delete_forward(),
                        2 => self.view_states.income.form.amount.delete_forward(),
                        _ => {}
                    },
                    KeyCode::Char(c) => match self.view_states.income.form.active_field {
                        0 => self.view_states.income.form.var_name.insert(c),
                        1 => self.view_states.income.form.display_name.insert(c),
                        2 => self.view_states.income.form.amount.insert(c),
                        _ => {}
                    },
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
                        let amount = self
                            .view_states
                            .expenses
                            .form
                            .amount
                            .value()
                            .parse::<Decimal>()
                            .unwrap_or(Decimal::ZERO);

                        let service = crate::services::ExpenseService::new(&self.db);
                        let result = if self.view_states.expenses.form.is_edit {
                            if let Some(id_str) = &self.view_states.expenses.form.edit_id {
                                if let Ok(Some(mut source)) = service.get_by_id(id_str) {
                                    source.variable_name =
                                        self.view_states.expenses.form.var_name.value().to_string();
                                    source.display_name = self
                                        .view_states
                                        .expenses
                                        .form
                                        .display_name
                                        .value()
                                        .to_string();
                                    source.amount = amount;
                                    source.expense_type =
                                        crate::ui::views::expenses::expense_types()
                                            [self.view_states.expenses.form.type_idx]
                                            .clone();
                                    source.frequency = crate::ui::views::income::frequencies()
                                        [self.view_states.expenses.form.frequency_idx]
                                        .clone();
                                    source.category =
                                        crate::ui::views::expenses::expense_categories()
                                            [self.view_states.expenses.form.category_idx]
                                            .clone();

                                    service.update(&source)
                                } else {
                                    Err(crate::error::CashCraftError::Validation(
                                        "Record not found".to_string(),
                                    ))
                                }
                            } else {
                                Err(crate::error::CashCraftError::Validation(
                                    "Missing edit ID".to_string(),
                                ))
                            }
                        } else {
                            let source = crate::domain::expense::Expense::new(
                                self.view_states.expenses.form.var_name.value().to_string(),
                                self.view_states
                                    .expenses
                                    .form
                                    .display_name
                                    .value()
                                    .to_string(),
                                amount,
                                crate::ui::views::expenses::expense_types()
                                    [self.view_states.expenses.form.type_idx]
                                    .clone(),
                                crate::ui::views::income::frequencies()
                                    [self.view_states.expenses.form.frequency_idx]
                                    .clone(),
                                crate::ui::views::expenses::expense_categories()
                                    [self.view_states.expenses.form.category_idx]
                                    .clone(),
                            );
                            service.create(&source)
                        };

                        if let Err(e) = result {
                            self.view_states.expenses.form.error = Some(format!("Error: {}", e));
                        } else {
                            self.view_states.refresh_current(View::Expenses, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.expenses.form.is_open = false;
                            let msg = if self.view_states.expenses.form.is_edit {
                                "Expense updated"
                            } else {
                                "Expense added"
                            };
                            self.app.set_success(msg);
                        }
                    }
                    KeyCode::Tab => {
                        self.view_states.expenses.form.active_field =
                            (self.view_states.expenses.form.active_field + 1) % 6;
                    }
                    KeyCode::BackTab => {
                        self.view_states.expenses.form.active_field =
                            (self.view_states.expenses.form.active_field + 5) % 6;
                    }
                    KeyCode::Left => match self.view_states.expenses.form.active_field {
                        3 => {
                            let idx = self.view_states.expenses.form.type_idx;
                            self.view_states.expenses.form.type_idx =
                                if idx == 0 { 2 } else { idx - 1 };
                        }
                        4 => {
                            let idx = self.view_states.expenses.form.frequency_idx;
                            self.view_states.expenses.form.frequency_idx =
                                if idx == 0 { 6 } else { idx - 1 };
                        }
                        5 => {
                            let len = crate::ui::views::expenses::expense_categories().len();
                            let idx = self.view_states.expenses.form.category_idx;
                            self.view_states.expenses.form.category_idx =
                                if idx == 0 { len - 1 } else { idx - 1 };
                        }
                        0 => self.view_states.expenses.form.var_name.move_left(),
                        1 => self.view_states.expenses.form.display_name.move_left(),
                        2 => self.view_states.expenses.form.amount.move_left(),
                        _ => {}
                    },
                    KeyCode::Right => match self.view_states.expenses.form.active_field {
                        3 => {
                            self.view_states.expenses.form.type_idx =
                                (self.view_states.expenses.form.type_idx + 1) % 3;
                        }
                        4 => {
                            self.view_states.expenses.form.frequency_idx =
                                (self.view_states.expenses.form.frequency_idx + 1) % 7;
                        }
                        5 => {
                            let len = crate::ui::views::expenses::expense_categories().len();
                            self.view_states.expenses.form.category_idx =
                                (self.view_states.expenses.form.category_idx + 1) % len;
                        }
                        0 => self.view_states.expenses.form.var_name.move_right(),
                        1 => self.view_states.expenses.form.display_name.move_right(),
                        2 => self.view_states.expenses.form.amount.move_right(),
                        _ => {}
                    },
                    KeyCode::Backspace => match self.view_states.expenses.form.active_field {
                        0 => self.view_states.expenses.form.var_name.delete(),
                        1 => self.view_states.expenses.form.display_name.delete(),
                        2 => self.view_states.expenses.form.amount.delete(),
                        _ => {}
                    },
                    KeyCode::Delete => match self.view_states.expenses.form.active_field {
                        0 => self.view_states.expenses.form.var_name.delete_forward(),
                        1 => self.view_states.expenses.form.display_name.delete_forward(),
                        2 => self.view_states.expenses.form.amount.delete_forward(),
                        _ => {}
                    },
                    KeyCode::Char(c) => match self.view_states.expenses.form.active_field {
                        0 => self.view_states.expenses.form.var_name.insert(c),
                        1 => self.view_states.expenses.form.display_name.insert(c),
                        2 => self.view_states.expenses.form.amount.insert(c),
                        _ => {}
                    },
                    _ => {}
                }
            }
            View::Transactions => {
                if !self.view_states.transactions.form.is_open {
                    self.app.enter_normal_mode();
                    return;
                }

                // Check if autocomplete is active on category field
                let is_category_field = self.view_states.transactions.form.active_field == 4;
                let autocomplete_visible = self
                    .view_states
                    .transactions
                    .form
                    .category_autocomplete
                    .visible;

                match key.code {
                    KeyCode::Enter => {
                        // If autocomplete is visible and has selection, accept it first
                        if is_category_field && autocomplete_visible {
                            if let Some(selected) = self
                                .view_states
                                .transactions
                                .form
                                .category_autocomplete
                                .accept()
                            {
                                self.view_states
                                    .transactions
                                    .form
                                    .category
                                    .set_value(selected);
                            }
                            return;
                        }

                        // Otherwise save the form
                        use rust_decimal::Decimal;
                        let amount = self
                            .view_states
                            .transactions
                            .form
                            .amount
                            .value()
                            .parse::<Decimal>()
                            .unwrap_or(Decimal::ZERO);

                        let date_str = self.view_states.transactions.form.date.value();
                        let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                            .unwrap_or_else(|_| chrono::Local::now().date_naive());

                        let service = crate::services::TransactionService::new(&self.db);
                        let result = if self.view_states.transactions.form.is_edit {
                            if let Some(id_str) = &self.view_states.transactions.form.edit_id {
                                if let Ok(Some(mut source)) = service.get_by_id(id_str) {
                                    source.date = date;
                                    source.description = self
                                        .view_states
                                        .transactions
                                        .form
                                        .description
                                        .value()
                                        .to_string();
                                    source.amount = amount;
                                    source.transaction_type =
                                        crate::ui::views::transactions::transaction_types()
                                            [self.view_states.transactions.form.type_idx]
                                            .clone();
                                    source.category = self
                                        .view_states
                                        .transactions
                                        .form
                                        .category
                                        .value()
                                        .to_string();

                                    service.update(&source)
                                } else {
                                    Err(crate::error::CashCraftError::Validation(
                                        "Record not found".to_string(),
                                    ))
                                }
                            } else {
                                Err(crate::error::CashCraftError::Validation(
                                    "Missing edit ID".to_string(),
                                ))
                            }
                        } else {
                            let source = crate::domain::transaction::Transaction::new(
                                date,
                                self.view_states
                                    .transactions
                                    .form
                                    .description
                                    .value()
                                    .to_string(),
                                amount,
                                crate::ui::views::transactions::transaction_types()
                                    [self.view_states.transactions.form.type_idx]
                                    .clone(),
                                self.view_states
                                    .transactions
                                    .form
                                    .category
                                    .value()
                                    .to_string(),
                            );
                            service.create(&source)
                        };

                        if let Err(e) = result {
                            self.view_states.transactions.form.error =
                                Some(format!("Error: {}", e));
                        } else {
                            self.view_states
                                .refresh_current(View::Transactions, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.transactions.form.is_open = false;
                            let msg = if self.view_states.transactions.form.is_edit {
                                "Transaction updated"
                            } else {
                                "Transaction added"
                            };
                            self.app.set_success(msg);
                        }
                    }
                    KeyCode::Tab => {
                        // Hide autocomplete when moving to next field
                        self.view_states
                            .transactions
                            .form
                            .category_autocomplete
                            .hide();
                        self.view_states.transactions.form.active_field =
                            (self.view_states.transactions.form.active_field + 1) % 5;
                    }
                    KeyCode::BackTab => {
                        self.view_states
                            .transactions
                            .form
                            .category_autocomplete
                            .hide();
                        self.view_states.transactions.form.active_field =
                            (self.view_states.transactions.form.active_field + 4) % 5;
                    }
                    KeyCode::Down => {
                        if is_category_field {
                            self.view_states
                                .transactions
                                .form
                                .category_autocomplete
                                .select_next();
                        }
                    }
                    KeyCode::Up => {
                        if is_category_field {
                            self.view_states
                                .transactions
                                .form
                                .category_autocomplete
                                .select_prev();
                        }
                    }
                    KeyCode::Left => match self.view_states.transactions.form.active_field {
                        3 => {
                            let idx = self.view_states.transactions.form.type_idx;
                            self.view_states.transactions.form.type_idx =
                                if idx == 0 { 2 } else { idx - 1 };
                        }
                        0 => self.view_states.transactions.form.date.move_left(),
                        1 => self.view_states.transactions.form.description.move_left(),
                        2 => self.view_states.transactions.form.amount.move_left(),
                        4 => self.view_states.transactions.form.category.move_left(),
                        _ => {}
                    },
                    KeyCode::Right => match self.view_states.transactions.form.active_field {
                        3 => {
                            self.view_states.transactions.form.type_idx =
                                (self.view_states.transactions.form.type_idx + 1) % 3;
                        }
                        0 => self.view_states.transactions.form.date.move_right(),
                        1 => self.view_states.transactions.form.description.move_right(),
                        2 => self.view_states.transactions.form.amount.move_right(),
                        4 => self.view_states.transactions.form.category.move_right(),
                        _ => {}
                    },
                    KeyCode::Backspace => {
                        match self.view_states.transactions.form.active_field {
                            0 => self.view_states.transactions.form.date.delete(),
                            1 => self.view_states.transactions.form.description.delete(),
                            2 => self.view_states.transactions.form.amount.delete(),
                            4 => {
                                self.view_states.transactions.form.category.delete();
                                // Update autocomplete filter
                                let value = self
                                    .view_states
                                    .transactions
                                    .form
                                    .category
                                    .value()
                                    .to_string();
                                self.view_states
                                    .transactions
                                    .form
                                    .category_autocomplete
                                    .filter(&value);
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Delete => match self.view_states.transactions.form.active_field {
                        0 => self.view_states.transactions.form.date.delete_forward(),
                        1 => self
                            .view_states
                            .transactions
                            .form
                            .description
                            .delete_forward(),
                        2 => self.view_states.transactions.form.amount.delete_forward(),
                        4 => {
                            self.view_states.transactions.form.category.delete_forward();
                            // Update autocomplete filter
                            let value = self
                                .view_states
                                .transactions
                                .form
                                .category
                                .value()
                                .to_string();
                            self.view_states
                                .transactions
                                .form
                                .category_autocomplete
                                .filter(&value);
                        }
                        _ => {}
                    },
                    KeyCode::Char(c) => {
                        match self.view_states.transactions.form.active_field {
                            0 => self.view_states.transactions.form.date.insert(c),
                            1 => self.view_states.transactions.form.description.insert(c),
                            2 => self.view_states.transactions.form.amount.insert(c),
                            4 => {
                                self.view_states.transactions.form.category.insert(c);
                                // Update autocomplete filter
                                let value = self
                                    .view_states
                                    .transactions
                                    .form
                                    .category
                                    .value()
                                    .to_string();
                                self.view_states
                                    .transactions
                                    .form
                                    .category_autocomplete
                                    .filter(&value);
                            }
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

                // Check if autocomplete is active on category field
                let is_category_field = self.view_states.budget.form.active_field == 0;
                let autocomplete_visible =
                    self.view_states.budget.form.category_autocomplete.visible;

                match key.code {
                    KeyCode::Enter => {
                        // If autocomplete is visible and has selection, accept it first
                        if is_category_field && autocomplete_visible {
                            if let Some(selected) =
                                self.view_states.budget.form.category_autocomplete.accept()
                            {
                                self.view_states.budget.form.category.set_value(selected);
                            }
                            return;
                        }

                        // Otherwise save the form
                        use rust_decimal::Decimal;
                        let amount = self
                            .view_states
                            .budget
                            .form
                            .amount
                            .value()
                            .parse::<Decimal>()
                            .unwrap_or(Decimal::ZERO);
                        let category = self.view_states.budget.form.category.value().to_string();

                        let month = self.view_states.budget.month;
                        let year = self.view_states.budget.year;
                        let is_template_mode = self.view_states.budget.form.is_template_mode;

                        let service = crate::services::BudgetService::new(&self.db);
                        let result = if self.view_states.budget.form.is_edit {
                            if let Some(id_str) = &self.view_states.budget.form.edit_id {
                                if let Ok(Some(mut source)) = service.get_by_id(id_str) {
                                    source.category = category;
                                    source.amount = amount;
                                    // Preserve the original is_template status
                                    service.update(&source)
                                } else {
                                    Err(crate::error::CashCraftError::Validation(
                                        "Record not found".to_string(),
                                    ))
                                }
                            } else {
                                Err(crate::error::CashCraftError::Validation(
                                    "Missing edit ID".to_string(),
                                ))
                            }
                        } else if is_template_mode {
                            // Create a template (applies to all months)
                            service.create_template(&category, amount).map(|_| ())
                        } else {
                            // Create an override for this specific month
                            service
                                .create_override(year, month, &category, amount)
                                .map(|_| ())
                        };

                        if let Err(e) = result {
                            self.view_states.budget.form.error = Some(format!("Error: {}", e));
                        } else {
                            self.view_states.refresh_current(View::Budget, &self.db);
                            self.app.enter_normal_mode();
                            self.view_states.budget.form.is_open = false;
                            let msg = if self.view_states.budget.form.is_edit {
                                "Budget updated"
                            } else if is_template_mode {
                                "Template created (applies to all months)"
                            } else {
                                "Override created (this month only)"
                            };
                            self.app.set_success(msg);
                        }
                    }
                    KeyCode::Tab => {
                        // Hide autocomplete when moving to next field
                        self.view_states.budget.form.category_autocomplete.hide();
                        self.view_states.budget.form.active_field =
                            (self.view_states.budget.form.active_field + 1) % 2;
                    }
                    KeyCode::BackTab => {
                        self.view_states.budget.form.category_autocomplete.hide();
                        self.view_states.budget.form.active_field =
                            (self.view_states.budget.form.active_field + 1) % 2;
                    }
                    KeyCode::Down => {
                        if is_category_field {
                            self.view_states
                                .budget
                                .form
                                .category_autocomplete
                                .select_next();
                        }
                    }
                    KeyCode::Up => {
                        if is_category_field {
                            self.view_states
                                .budget
                                .form
                                .category_autocomplete
                                .select_prev();
                        }
                    }
                    KeyCode::Left => match self.view_states.budget.form.active_field {
                        0 => self.view_states.budget.form.category.move_left(),
                        1 => self.view_states.budget.form.amount.move_left(),
                        _ => {}
                    },
                    KeyCode::Right => match self.view_states.budget.form.active_field {
                        0 => self.view_states.budget.form.category.move_right(),
                        1 => self.view_states.budget.form.amount.move_right(),
                        _ => {}
                    },
                    KeyCode::Backspace => {
                        match self.view_states.budget.form.active_field {
                            0 => {
                                self.view_states.budget.form.category.delete();
                                // Update autocomplete filter
                                let value =
                                    self.view_states.budget.form.category.value().to_string();
                                self.view_states
                                    .budget
                                    .form
                                    .category_autocomplete
                                    .filter(&value);
                            }
                            1 => self.view_states.budget.form.amount.delete(),
                            _ => {}
                        }
                    }
                    KeyCode::Delete => {
                        match self.view_states.budget.form.active_field {
                            0 => {
                                self.view_states.budget.form.category.delete_forward();
                                // Update autocomplete filter
                                let value =
                                    self.view_states.budget.form.category.value().to_string();
                                self.view_states
                                    .budget
                                    .form
                                    .category_autocomplete
                                    .filter(&value);
                            }
                            1 => self.view_states.budget.form.amount.delete_forward(),
                            _ => {}
                        }
                    }
                    KeyCode::Char(c) => {
                        match self.view_states.budget.form.active_field {
                            0 => {
                                self.view_states.budget.form.category.insert(c);
                                // Update autocomplete filter
                                let value =
                                    self.view_states.budget.form.category.value().to_string();
                                self.view_states
                                    .budget
                                    .form
                                    .category_autocomplete
                                    .filter(&value);
                            }
                            1 => self.view_states.budget.form.amount.insert(c),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            View::Playground => match key.code {
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
            },
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

    if app.show_confirmation {
        draw_confirmation_modal(frame, app);
    }
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
            View::Transactions => "j/k:nav  a:add  /:search  [/]:month  q:quit",
            View::Budget => "j/k:nav  a:add(template)  o:override  e:edit  [/]:month  q:quit",
            View::Playground => "Enter:eval  Ctrl-C:clear  q:quit",
            View::Charts => "h/l:type  q:quit",
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

/// Draw confirmation modal
fn draw_confirmation_modal(frame: &mut Frame, app: &App) {
    let theme = &app.theme;

    // Get message based on pending action
    let message = if let Some(action) = &app.pending_action {
        match action {
            PendingAction::DeleteIncome(_) => "Are you sure you want to delete this income source?",
            PendingAction::DeleteExpense(_) => "Are you sure you want to delete this expense?",
            PendingAction::DeleteTransaction(_) => {
                "Are you sure you want to delete this transaction?"
            }
            PendingAction::DeleteBudget(_) => "Are you sure you want to delete this budget?",
        }
    } else {
        "Are you sure?"
    };

    let block = Block::default()
        .title(Span::styled(
            " Confirmation ",
            Style::default()
                .fg(theme.colors.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.colors.warning))
        .style(Style::default().bg(theme.colors.surface));

    let text = vec![
        Line::from(Span::styled(
            message,
            Style::default().fg(theme.colors.text_primary),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "(y) Yes",
                Style::default()
                    .fg(theme.colors.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(
                "(n) No",
                Style::default()
                    .fg(theme.colors.error)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    let area = crate::ui::layout::modal(frame.area(), 60, 20);
    frame.render_widget(ratatui::widgets::Clear, area); // Clear underlying content
    frame.render_widget(paragraph, area);
}
