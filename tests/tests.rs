use std::sync::Arc;
use test_bot::env_utils::EnvParams;
use test_bot::tx::Transaction;

#[derive(Debug)]
pub struct TestTransaction {
    pub(crate) wallet: String,
    pub(crate) token: String,
    pub(crate) adjusted_commission: i64,
    pub(crate) price: i64,
}

impl TestTransaction {
    pub fn new_stable_min(params: &Arc<EnvParams>) -> Self {
        let adjusted_commission = params.commission - params.commission_change;
        Self {
            wallet: params.wallet.clone(),
            token: params.token.clone(),
            adjusted_commission,
            price: params.price,
        }
    }
}

impl Transaction for TestTransaction {
    fn amount(&self) -> i64 {
        self.adjusted_commission + self.price
    }

    fn execute(&self) -> Result<String, String> {
        Ok(self.info().to_string())
    }

    fn info(&self) -> String {
        format!(
            "\nWallet: {}, Token: {}, Commission: {}, Price: {}, Amount: {}",
            self.wallet,
            self.token,
            self.adjusted_commission,
            self.price,
            self.amount()
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::*;
    use test_bot::limits::{LimitChecker, States};

    // Тесты
    #[test]
    fn test_transaction_exceeds_limit_and_finishes() {
        let params = Arc::new(EnvParams {
            wallet: "test_wallet".to_string(),
            token: "test_token".to_string(),
            total_amount: 189,
            commission: 100,
            commission_change: 10,
            max_transactions: 100,
            max_threads: 1,
            price: 100,
        });

        let tx = TestTransaction::new_stable_min(&params);

        let limiter = LimitChecker::new(&params);
        let result = limiter.process_transaction(&tx);

        assert!(matches!(result, Ok(States::Finish)));
        assert_eq!(limiter.transactions_count.load(Ordering::SeqCst), 0);
        assert_eq!(limiter.current_amount.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_successful_transaction() {
        let params = Arc::new(EnvParams {
            wallet: "test_wallet".to_string(),
            token: "test_token".to_string(),
            total_amount: 211,
            commission: 100,
            commission_change: 10,
            max_transactions: 100,
            max_threads: 1,
            price: 100,
        });

        let tx = TestTransaction::new_stable_min(&params);
        let limiter = LimitChecker::new(&params);

        let result = limiter.process_transaction(&tx);
        assert!(matches!(result, Ok(States::InProgres(_))));
    }
}
