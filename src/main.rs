use limits::{LimitChecker, States};
use log::{info, warn};
use rayon::ThreadPoolBuilder;
use std::sync::{Arc, Mutex};
use tx_genertor::TransactionGenerator;

pub mod env_utils;
mod limits;
mod tx;
mod tx_genertor;

/// Initializes and starts the bot for processing transactions.
fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let params = Arc::new(env_utils::EnvParams::read_env());
    info!("Starting bot with parameters: {:?}", &params);

    let limiter = LimitChecker::new(&params);
    let generator = TransactionGenerator::new(params.clone());
    let results = Arc::new(Mutex::new(Vec::new()));

    run_transaction_process(params.max_threads, generator, limiter.into(), &results);

    // Retrieve and display the results
    let final_results = unwrap_results(results);
    display_results(final_results);
}

/// Runs the multi-threaded transaction processing.
///
/// # Arguments
/// * `max_threads` - The maximum number of threads.
/// * `generator` - The transaction generator.
/// * `limiter` - The limit checker.
/// * `results` - Arc wrapper around Mutex for collecting results.
fn run_transaction_process(
    max_threads: usize,
    generator: TransactionGenerator,
    limiter: Arc<LimitChecker>,
    results: &Arc<Mutex<Vec<States>>>,
) {
    let pool = ThreadPoolBuilder::new()
        .num_threads(max_threads)
        .build()
        .expect("Failed to create thread pool");

    pool.install(|| {
        let local_results: Vec<_> = generator
            .into_iter()
            .map_while(|tx| match limiter.process_transaction(&tx) {
                Ok(state) if state != States::Finish => Some(state),
                _ => None,
            })
            .collect();

        let mut global_results = results.lock().unwrap();
        global_results.extend(local_results);
    });
}

/// Extracts results from the shared storage and returns them.
///
/// # Arguments
/// * `results` - Arc wrapper around Mutex for collecting results.
fn unwrap_results(results: Arc<Mutex<Vec<States>>>) -> Vec<States> {
    match Arc::try_unwrap(results) {
        Ok(mutex) => mutex.into_inner().unwrap_or_else(|_| {
            warn!("Failed to lock mutex, returning empty results");
            Vec::new()
        }),
        Err(_) => {
            warn!("Arc still has multiple owners, returning empty results");
            Vec::new()
        }
    }
}

/// Displays the transaction results (signatures) in the console with numbering.
///
/// # Arguments
/// * `results` - A vector of `States` containing the transaction states.
fn display_results(results: Vec<States>) {
    info!("Transaction Signatures:");
    for (index, state) in results.into_iter().enumerate() {
        if let States::InProgres(signature) = state {
            println!("{}. {}", index + 1, signature);
        }
    }
}
