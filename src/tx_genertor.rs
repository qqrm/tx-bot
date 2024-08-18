use std::sync::Arc;

use crate::env_utils::EnvParams;
use crate::tx::SomeTransaction;

use derive_builder::Builder;
use SomeTransaction as Transaction;

/// A generator that creates an infinite stream of transactions
/// using the specified parameters.
#[derive(Default, derive_new::new, Builder)]
pub(crate) struct TransactionGenerator {
    /// Environment parameters containing information about the wallet, token, etc.
    pub(crate) params: Arc<EnvParams>,
}

impl Iterator for TransactionGenerator {
    type Item = Transaction;

    /// Returns the next transaction in the sequence.
    ///
    /// # Returns
    /// `Option<Transaction>` - A new transaction based on the current parameters.
    fn next(&mut self) -> Option<Self::Item> {
        let tx = Transaction::new(&self.params);
        Some(tx)
    }
}
