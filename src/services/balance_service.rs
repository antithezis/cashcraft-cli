use crate::error::Result;
use crate::repository::{BalanceRepository, TransactionRepository};
use rust_decimal::Decimal;

pub struct BalanceService<'a> {
    balance_repo: &'a BalanceRepository<'a>,
    transaction_repo: &'a TransactionRepository<'a>,
}

impl<'a> BalanceService<'a> {
    pub fn new(
        balance_repo: &'a BalanceRepository<'a>,
        transaction_repo: &'a TransactionRepository<'a>,
    ) -> Self {
        Self {
            balance_repo,
            transaction_repo,
        }
    }

    pub fn get_opening_balance(&self, year: i32, month: u32) -> Result<Decimal> {
        // 1. Check if there's an explicit opening balance for this month
        if let Some(balance) = self.balance_repo.get(year, month)? {
            return Ok(balance.amount);
        }

        // 2. Iteratively calculate from the latest available balance
        let mut current_year = year;
        let mut current_month = month;

        // Find the base month (latest month < current with an override)
        // We look back up to 5 years. If nothing found, we start from 0 at a reasonable start date.
        let mut base_year = year;
        let mut base_month = month;
        let mut base_amount = Decimal::ZERO;
        let mut found_base = false;

        // Limit lookback to avoid infinite loops
        for _ in 0..60 {
            // 5 years
            // Move to previous month
            if current_month == 1 {
                current_year -= 1;
                current_month = 12;
            } else {
                current_month -= 1;
            }

            if let Some(balance) = self.balance_repo.get(current_year, current_month)? {
                base_year = current_year;
                base_month = current_month;
                base_amount = balance.amount;
                found_base = true;
                break;
            }
        }

        if !found_base {
            // If no base found, start from the beginning of time (or a reasonable default)
            // Ideally, we should find the first transaction.
            // For simplicity, let's assume we calculate from 5 years ago if no override exists.
            // Or better, just calculate from the current lookback point (year - 5).
            base_year = current_year;
            base_month = current_month;
            // base_amount is 0
        }

        // Now calculate forward from base to target
        self.calculate_balance_forward(base_year, base_month, base_amount, year, month)
    }

    fn calculate_balance_forward(
        &self,
        start_year: i32,
        start_month: u32,
        start_amount: Decimal,
        target_year: i32,
        target_month: u32,
    ) -> Result<Decimal> {
        let mut balance = start_amount;
        let mut y = start_year;
        let mut m = start_month;

        // Loop until we reach the target month
        while y < target_year || (y == target_year && m < target_month) {
            // Add net income for month (y, m)
            let transactions = self.transaction_repo.get_by_month(y, m)?;
            for t in transactions {
                match t.transaction_type {
                    crate::domain::TransactionType::Income => {
                        balance += t.amount;
                    }
                    crate::domain::TransactionType::Expense => {
                        balance -= t.amount;
                    }
                    // Transfers might affect balance if we track accounts, but for overall budget,
                    // a transfer is usually neutral unless it's off-budget.
                    // Assuming single pot for now as per likely schema.
                    crate::domain::TransactionType::Transfer => {}
                }
            }

            // Move to next month
            if m == 12 {
                y += 1;
                m = 1;
            } else {
                m += 1;
            }
        }

        Ok(balance)
    }

    pub fn set_opening_balance(&self, year: i32, month: u32, amount: Decimal) -> Result<()> {
        self.balance_repo.set(year, month, amount)
    }
}
