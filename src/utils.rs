use dotenv::dotenv;
use std::env;

/// Macro to fetch and convert an environment variable to a specified type.
/// Panics if the variable is not set or cannot be converted to the specified type.
macro_rules! get_env {
    ($var:expr, $typ:ty) => {
        env::var($var)
            .unwrap_or_else(|_| panic!("{} not set", $var))
            .parse::<$typ>()
            .unwrap_or_else(|_| panic!("{} should be a {}", $var, stringify!($typ)))
    };
}

/// Environmental parameters for configuring the transaction bot.
///
/// # Parameters
/// * `wallet` - The blockchain wallet address from which transactions will be initiated.
/// * `token` - The specific token to be purchased in transactions.
/// * `total_amount` - The target total amount to be spent on token purchases.
/// * `max_transactions` - The maximum number of transactions to attempt.
/// * `commission` - Base commission for transactions, will vary +/- `commission_change`.
/// * `commission_change` - Allowed variation in commission, to be added or subtracted randomly.
/// * `max_threads` - The maximum number of concurrent threads for sending transactions.
///
#[derive(Debug, Clone)]
pub(crate) struct EnvParams {
    pub(crate) wallet: String,
    pub(crate) token: String,
    pub(crate) total_amount: i64,
    pub(crate) max_transactions: usize,
    pub(crate) commission: i64,
    pub(crate) commission_change: i64,
    pub(crate) max_threads: usize,
}

impl EnvParams {
    /// Reads and parses environment variables, creating a new instance of `EnvParams`.
    ///
    /// # Panics
    /// Panics if any environment variable is not set or cannot be parsed into the expected type.
    pub(crate) fn read_env() -> Self {
        dotenv().ok();

        Self {
            wallet: get_env!("WALLET", String),
            token: get_env!("TOKEN", String),
            total_amount: get_env!("TOTAL_AMOUNT", i64),
            commission: get_env!("COMMISSION", i64),
            commission_change: get_env!("COMMISSION_CHANGE", i64),
            max_transactions: get_env!("MAX_TRANSACTIONS", usize),
            max_threads: {
                let max_threads_env: usize = get_env!("MAX_THREADS", usize);
                std::cmp::min(num_cpus::get(), max_threads_env)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use serial_test::serial;

    use super::*;

    fn setup_env() {
        env::set_var("WALLET", "TestWallet");
        env::set_var("TOKEN", "TestToken");
        env::set_var("TOTAL_AMOUNT", "1000");
        env::set_var("COMMISSION", "100");
        env::set_var("COMMISSION_CHANGE", "10");
        env::set_var("MAX_TRANSACTIONS", "50");
        env::set_var("MAX_THREADS", "4");
    }

    fn cleanup_env() {
        env::remove_var("WALLET");
        env::remove_var("TOKEN");
        env::remove_var("TOTAL_AMOUNT");
        env::remove_var("COMMISSION");
        env::remove_var("COMMISSION_CHANGE");
        env::remove_var("MAX_TRANSACTIONS");
        env::remove_var("MAX_THREADS");
    }

    // Ensures cleanup after test completion (in case of panic)
    struct EnvironmentGuard;
 
    impl Drop for EnvironmentGuard {
        fn drop(&mut self) {
            cleanup_env();
        }
    }

    #[test]
    #[serial]
    fn test_read_env_correctly() {
        let _env_guard = EnvironmentGuard; 
        setup_env();
        let params = EnvParams::read_env();
        assert_eq!(params.wallet, "TestWallet");
        assert_eq!(params.token, "TestToken");
        assert_eq!(params.total_amount, 1000);
        assert_eq!(params.commission, 100);
        assert_eq!(params.commission_change, 10);
        assert_eq!(params.max_transactions, 50);
        assert_eq!(params.max_threads, 4);
    }

    #[test]
    #[serial]
    #[should_panic(expected = "WALLET not set")]
    fn test_missing_wallet() {
        let _env_guard = EnvironmentGuard;
        cleanup_env();
        env::remove_var("WALLET");
        let _ = get_env!("WALLET", String);
    }

    #[test]
    #[serial]
    #[should_panic(expected = "TOTAL_AMOUNT should be a i64")]
    fn test_invalid_total_amount() {
        let _env_guard = EnvironmentGuard;
        cleanup_env();
        env::set_var("TOTAL_AMOUNT", "not_a_number");
        let _ = get_env!("TOTAL_AMOUNT", i64);
    }
}
