use clap::Parser;
use clap_derive::Parser;
use csv::Writer;
use tests::fuzzing::fuzz_target_operation::{ActionPoolChange, FuzzTargetOperation};
use tests::utils::{TestingEnv, TestingEnvConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(long, default_value = "50")]
    pub run_len: usize,
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
    let CliArgs { run_len } = CliArgs::parse();

    let operations = FuzzTargetOperation::generate_run(run_len);

    let testing_env =
        TestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(1_250_000.0));

    let mut run_result = RunResult::default();

    let mut wtr = Writer::from_path("random-walk.csv").unwrap();
    wtr.write_record(["d", "total_lp", "diff"]).unwrap();

    for (i, operation) in operations.iter().enumerate() {
        let operation_result = operation.execute(&testing_env);
        let operation_status = operation_result.is_ok();
        testing_env.pool.assert_total_lp_less_or_equal_d();

        run_result.update(operation, operation_status);

        println!(
            "\r {}/{}, operation: {:?}, status_ok: {}",
            i + 1,
            run_len,
            operation,
            operation_status
        );

        let total_lp = testing_env.pool.total_lp();
        let d = testing_env.pool.d();

        wtr.serialize(ActionPoolChange {
            total_lp: testing_env.pool.total_lp(),
            d: testing_env.pool.d(),
            diff: total_lp as i128 - d as i128,
        })
        .unwrap();
    }

    wtr.flush().unwrap();
    println!("{}", run_result.to_string());
}
