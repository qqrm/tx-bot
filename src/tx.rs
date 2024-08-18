use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use derive_builder::Builder;
use log::warn;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::env_utils::EnvParams;

/// Trait that defines a transaction. 
/// Implementations of this trait should define how to calculate the amount of the transaction, execute it, and provide info about it.
pub trait Transaction {
    /// Returns the total amount of the transaction (price + commission).
    fn amount(&self) -> i64;

    /// Executes the transaction. Returns the transaction signature as `Ok(String)` if successful,
    /// or an error message as `Err(String)` if the transaction fails.
    fn execute(&self) -> Result<String, String>;

    /// Returns information about the transaction in the form of a string.
    fn info(&self) -> String;
}

/// Struct representing a transaction with specific parameters such as wallet, token, adjusted commission, and price.
#[derive(Builder, Default, Debug)]
pub struct SomeTransaction {
    pub(crate) wallet: String,
    pub(crate) token: String,
    pub(crate) adjusted_commission: i64,
    pub(crate) price: i64,
}

impl SomeTransaction {
    /// Creates a new instance of `SomeTransaction`, adjusting the commission based on the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - A reference-counted pointer to `EnvParams` that contains the environment parameters for the transaction.
    pub fn new(params: &Arc<EnvParams>) -> Self {
        let mut rng = StdRng::from_entropy();
        let adjusted_commission =
            params.commission + rng.gen_range(-params.commission_change..=params.commission_change);

        Self {
            wallet: params.wallet.clone(),
            token: params.token.clone(),
            adjusted_commission,
            price: params.price,
        }
    }
}

impl Transaction for SomeTransaction {
    /// Returns the total amount of the transaction, which is the sum of the price and the adjusted commission.
    fn amount(&self) -> i64 {
        self.adjusted_commission + self.price
    }

    /// Executes the transaction. There is a small probability of failure determined by the current time in nanoseconds.
    /// If the transaction fails, it logs a warning and returns an error message. Otherwise, it returns the transaction information.
    fn execute(&self) -> Result<String, String> {
        let fail_condition = || {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
                % 10
                == 0
        };

        if fail_condition() {
            warn!("FAIL");
            Err("failed tx".to_string())
        } else {
            Ok(self.info())
        }
    }

    /// Provides a formatted string with details about the transaction, including the wallet, token, commission, price, and total amount.
    fn info(&self) -> String {
        format!(
            "Wallet: {}, Token: {}, Commission: {}, Price: {}, Amount: {}",
            self.wallet,
            self.token,
            self.adjusted_commission,
            self.price,
            self.amount()
        )
    }
}
