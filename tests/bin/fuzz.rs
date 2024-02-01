use std::fs::{self, OpenOptions};
use std::io::{stdout, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::Parser;
use clap_derive::Parser;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use soroban_sdk::Env;
use tabled::Table;

use tabled::settings::Style;
use tests::fuzzing::fuzz_target_operation::{Action, FuzzTargetOperation};
use tests::utils::{TestingEnvConfig, TestingEnvironment};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(long, default_value = "150")]
    pub runs: u64,
    #[arg(long, default_value = "50")]
    pub run_len: usize,
    #[arg(long, default_value = "4")]
    pub threads: usize,
    #[arg(short, long, default_value = "false")]
    pub stop_run_if_invariant_failed: bool,
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
        stop_run_if_invariant_failed: stop_if_invariant_failed,
    } = CliArgs::parse();

    let path_to_read = Path::new("fuzz-report.md");
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

    fs::write(path_to_read, "# ✨ Fuzz report ✨\n").expect("unable to write");

    runs.par_iter().for_each(|operations| {
        let env = Env::default();
        let testing_env = TestingEnvironment::create(
            &env,
            TestingEnvConfig::default()
                .with_yaro_admin_deposit(1_110_000.0)
                .with_yusd_admin_deposit(1_140_000.0),
        );

        let mut run_result = RunResult::default();
        let mut actions = Vec::with_capacity(run_len);

        for (i, operation) in operations.iter().enumerate() {
            let operation_result = operation.execute(&testing_env);
            let invariant_result = testing_env.pool.invariant_total_lp_less_or_equal_d();

            actions.push(Action {
                status: match operation_result {
                    Ok(_) => "✅",
                    Err(_) => "❌",
                },
                index: i + 1,
                log: operation.get_log_string(&operation_result),
                total_lp: testing_env.pool.total_lp(),
                d: testing_env.pool.d(),
                invariant: invariant_result.clone().err().unwrap_or("OK".into()),
            });

            run_result.update(&operation, operation_result.is_ok());

            if invariant_result.is_err() {
                eprintln!(
                    "\n\n{} at operation #{}",
                    invariant_result.err().unwrap().as_str(),
                    i + 1
                );

                if stop_if_invariant_failed {
                    break;
                }
            }
        }

        let mut current_run = successful_runs.lock().unwrap();
        *current_run += 1;

        let log = format!("{current_run}/{}. {}", runs.len(), &run_result.to_string());
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(path_to_read)
            .expect("unable to open file");
        let mut f = BufWriter::new(file);

        let mut table = Table::new(actions);
        table.with(Style::markdown());
        let table = table.to_string();

        writeln!(f, "{}\n\n{table}\n", log).expect("unable to write");

        stdout().flush().expect("Unable to flush stdout");
        print!("\r## {log}    ");
    });
}
