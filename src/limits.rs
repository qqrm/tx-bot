use std::fmt::Debug;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

use log::{debug, info};

use crate::env_utils::EnvParams;
use crate::tx::Transaction;

/// Enum representing the possible states of a transaction process.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum States {
    /// Indicates that the transaction process has finished.
    Finish,
    /// Indicates that the transaction is in progress, with a message (signature or error).
    InProgres(String),
}

/// Struct responsible for checking transaction limits and managing transaction counts and amounts.
#[derive(Debug)]
pub struct LimitChecker {
    /// Tracks the number of transactions processed.
    pub transactions_count: AtomicUsize,
    /// Tracks the current total amount processed in transactions.
    pub current_amount: AtomicI64,
    /// Stores the environment parameters for the transaction process.
    pub params: EnvParams,
}

impl LimitChecker {
    /// Creates a new `LimitChecker` instance.
    ///
    /// # Arguments
    ///
    /// * `params` - A reference to `EnvParams` containing the environment parameters.
    ///
    /// # Returns
    ///
    /// A new instance of `LimitChecker`.
    pub fn new(params: &EnvParams) -> Self {
        Self {
            transactions_count: AtomicUsize::new(0),
            current_amount: AtomicI64::new(0),
            params: params.clone(),
        }
    }

    /// Processes a transaction, checking limits and executing if within bounds.
    ///
    /// # Arguments
    ///
    /// * `tx` - A reference to the transaction to be processed.
    ///
    /// # Returns
    ///
    /// `Result<States, ()>` indicating the state after processing the transaction.
    pub fn process_transaction(&self, tx: &(impl Transaction + Debug)) -> Result<States, ()> {
        debug!("{}", tx.info());

        let tx_amount = tx.amount();

        // Check if there are sufficient funds for the transaction.
        if self.params.total_amount < tx_amount {
            info!("Insufficient funds for this transaction. Finishing process.");
            return Ok(States::Finish);
        }

        // Check if the transaction exceeds limits.
        if self.check(tx) {
            info!("Transaction within limits. Proceeding with execution.");

            self.transactions_count.fetch_add(1, Ordering::SeqCst);
            self.current_amount.fetch_add(tx_amount, Ordering::SeqCst);

            match tx.execute() {
                // Rollback counters if transaction execution fails.
                Err(err_mess) => {
                    info!("Transaction failed - rolling back counters.");

                    self.transactions_count.fetch_sub(1, Ordering::SeqCst);
                    self.current_amount.fetch_sub(tx_amount, Ordering::SeqCst);
                    Ok(States::InProgres(err_mess))
                }
                // Return success message if transaction execution succeeds.
                Ok(mess) => Ok(States::InProgres(mess)),
            }
        } else {
            info!("Transaction skipped: exceeds limits.");
            Ok(States::Finish)
        }
    }

    /// Checks if the transaction can be processed without exceeding limits.
    ///
    /// # Arguments
    ///
    /// * `tx` - A reference to the transaction to be checked.
    ///
    /// # Returns
    ///
    /// `bool` indicating whether the transaction can be processed.
    fn check(&self, tx: &impl Transaction) -> bool {
        let tx_amount = tx.amount();
        let transactions_count = self.transactions_count.load(Ordering::SeqCst);
        let current_amount = self.current_amount.load(Ordering::SeqCst);

        info!(
            "Checking transaction: transactions_count = {}, current_amount + tx_amount = {} (limit = {})",
            transactions_count, current_amount + tx_amount, self.params.total_amount
        );

        transactions_count < self.params.max_transactions
            && current_amount + tx_amount <= self.params.total_amount
    }
}

// Implementation of the Drop trait for `LimitChecker`.
impl Drop for LimitChecker {
    fn drop(&mut self) {
        let final_count = self.transactions_count.load(Ordering::SeqCst);
        let final_amount = self.current_amount.load(Ordering::SeqCst);
        info!(
            "LimitChecker is being dropped. Final transaction count: {}, Final total amount: {}",
            final_count, final_amount
        );
    }
}
