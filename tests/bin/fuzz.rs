use std::{
    io::{stdout, Write},
    sync::{Arc, Mutex},
};

use clap::Parser;
use clap_derive::Parser;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use tests::fuzzing::fuzz_target_operation::FuzzTargetOperation;
use tests::utils::{Snapshot, TestingEnvConfig, TestingEnv};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(long, default_value = "150")]
    pub runs: u64,
    #[arg(long, default_value = "50")]
    pub run_len: usize,
    #[arg(long, default_value = "4")]
    pub threads: usize,
}

#[derive(Debug, Clone, Default)]
struct RunResult {
    pub swaps: OperationResult,
    pub withdrawals: OperationResult,
    pub deposits: OperationResult,
}

impl RunResult {
    pub fn update(&mut self, operation: &FuzzTargetOperation, is_ok: bool) {
        let operation_result = match operation {
            FuzzTargetOperation::Swap { .. } => &mut self.swaps,
            FuzzTargetOperation::Deposit { .. } => &mut self.deposits,
            FuzzTargetOperation::Withdraw { .. } => &mut self.withdrawals,
        };

        operation_result.total += 1;
        operation_result.successful += is_ok as u32;
    }
}

impl ToString for RunResult {
    fn to_string(&self) -> String {
        format!(
            "💸 Swaps {}/{} | 📤 Deposits {}/{} | 📥 Withdrawals {}/{}",
            self.swaps.successful,
            self.swaps.total,
            self.deposits.successful,
            self.deposits.total,
            self.withdrawals.successful,
            self.withdrawals.total,
        )
    }
}

#[derive(Debug, Clone, Default)]
struct OperationResult {
    pub total: u32,
    pub successful: u32,
}

fn main() {
    let CliArgs {
        runs,
        run_len,
        threads,
    } = CliArgs::parse();

    let available_parallelism = std::thread::available_parallelism().unwrap().get();
    let successful_runs = Arc::new(Mutex::new(0));

    assert!(
        threads <= available_parallelism,
        "Available parallelism: {:?}",
        available_parallelism
    );

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    let runs = (0..runs)
        .into_par_iter()
        .map(|_| FuzzTargetOperation::generate_run(run_len))
        .collect::<Vec<_>>();

    runs.par_iter().for_each(|operations| {
        let testing_env = TestingEnv::create(
            TestingEnvConfig::default().with_admin_init_deposit(1_250_000.0),
        );

        let mut run_result = RunResult::default();

        let users_balance_sum_before = Snapshot::take(&testing_env).get_users_balances_sum();

        for operation in operations.iter() {
            let operation_result = operation.execute(&testing_env);

            run_result.update(operation, operation_result.is_ok());

            testing_env.pool.assert_total_lp_less_or_equal_d();
        }

        let users_balance_sum_after = Snapshot::take(&testing_env).get_users_balances_sum();

        assert!(
            users_balance_sum_after <= users_balance_sum_before,
            "Profit invariant"
        );

        let mut current_run = successful_runs.lock().unwrap();
        *current_run += 1;

        let log = format!("{current_run}/{}. {}", runs.len(), &run_result.to_string());

        stdout().flush().expect("Unable to flush stdout");
        print!("\r{log}    ");
    });
}
