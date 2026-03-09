use crate::domain::{
    budget::Budget, expense::Expense, income::IncomeSource, transaction::Transaction,
};

/// Represents an undoable action
#[derive(Debug, Clone)]
pub enum Action {
    DeleteIncome(IncomeSource),
    DeleteExpense(Expense),
    DeleteTransaction(Transaction),
    DeleteBudget(Budget),
    // Edit actions can be added later
}

/// Manages undo/redo history
#[derive(Debug, Default)]
pub struct History {
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, action: Action) {
        self.undo_stack.push(action);
        self.redo_stack.clear(); // Clear redo stack on new action
    }

    pub fn pop_undo(&mut self) -> Option<Action> {
        self.undo_stack.pop()
    }

    pub fn push_redo(&mut self, action: Action) {
        self.redo_stack.push(action);
    }

    pub fn pop_redo(&mut self) -> Option<Action> {
        self.redo_stack.pop()
    }

    pub fn push_undo_only(&mut self, action: Action) {
        self.undo_stack.push(action);
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
