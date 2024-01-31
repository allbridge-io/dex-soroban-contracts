use std::fs::{self, OpenOptions};
use std::io::{stdout, BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::Parser;
use clap_derive::Parser;
use rand::Rng;
use rand_derive2::RandGen;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use soroban_sdk::Env;

use tests::contracts::pool::Direction;
use tests::utils::{CallResult, TestingEnvConfig, TestingEnvironment, Token, User};

#[derive(Debug, Clone, Copy, RandGen)]
pub enum SwapDirection {
    YusdToYaro,
    YaroToYusd,
}

impl Into<Direction> for SwapDirection {
    fn into(self) -> Direction {
        match self {
            SwapDirection::YusdToYaro => Direction::A2B,
            SwapDirection::YaroToYusd => Direction::B2A,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, RandGen)]
pub enum UserID {
    Alice,
    Bob,
}

#[derive(Debug, RandGen)]
pub enum FuzzTargetOperation {
    Swap {
        direction: SwapDirection,
        amount: u16,
        sender: UserID,
        recipient: UserID,
    },
    Withdraw {
        lp_amount: u16,
        user: UserID,
    },
    Deposit {
        yusd_amount: u16,
        yaro_amount: u16,
        user: UserID,
    },
}

impl ToString for FuzzTargetOperation {
    fn to_string(&self) -> String {
        match self {
            FuzzTargetOperation::Swap {
                direction,
                amount,
                sender,
                recipient,
            } => {
                format!(
                    "[Swap] {} {:?}, from {:?} to {:?}",
                    amount, direction, sender, recipient
                )
            }

            FuzzTargetOperation::Deposit {
                yaro_amount,
                yusd_amount,
                user,
            } => {
                format!(
                    "[Deposit] {} YARO {} YUSD, user: {:?}",
                    yaro_amount, yusd_amount, user
                )
            }

            FuzzTargetOperation::Withdraw { lp_amount, user } => {
                format!("[Withdraw] {} lp, user: {:?}", lp_amount, user)
            }
        }
    }
}

impl FuzzTargetOperation {
    fn get_user(user_id: UserID, testing_env: &TestingEnvironment) -> &User {
        match user_id {
            UserID::Alice => &testing_env.alice,
            UserID::Bob => &testing_env.bob,
        }
    }

    fn get_tokens(direction: Direction, testing_env: &TestingEnvironment) -> (&Token, &Token) {
        match direction {
            Direction::A2B => (&testing_env.yusd_token, &testing_env.yaro_token),
            Direction::B2A => (&testing_env.yaro_token, &testing_env.yusd_token),
        }
    }

    pub fn execute(&self, testing_env: &TestingEnvironment) -> CallResult<()> {
        match self {
            FuzzTargetOperation::Swap {
                direction,
                amount,
                sender,
                recipient,
            } => {
                let sender = Self::get_user(*sender, testing_env);
                let recipient = Self::get_user(*recipient, testing_env);
                let amount = (*amount) as f64;
                let direction: Direction = (*direction).into();
                let (token_from, _) = Self::get_tokens(direction.clone(), testing_env);

                if token_from.balance_of_f64(sender.as_ref()) - amount <= 0.0 {
                    return Ok(());
                }

                testing_env
                    .pool
                    .swap(sender, recipient, amount, 0.0, direction)
                    .map(|_| ())
            }

            FuzzTargetOperation::Deposit {
                yaro_amount,
                yusd_amount,
                user,
            } => {
                let sender = Self::get_user(*user, testing_env);
                let deposits = (*yusd_amount as f64, *yaro_amount as f64);
                if deposits.0 + deposits.1 == 0.0 {
                    return Ok(());
                }

                testing_env.pool.deposit(sender, deposits, 0.0)
            }

            FuzzTargetOperation::Withdraw { lp_amount, user } => {
                let sender = Self::get_user(*user, testing_env);
                let lp_amount = (*lp_amount) as f64;

                testing_env.pool.withdraw(sender, lp_amount)
            }
        }
    }
}

pub fn generate_run(len: usize) -> Vec<FuzzTargetOperation> {
    let mut rng = rand::thread_rng();

    (0..len).into_iter().map(|_| rng.gen()).collect()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
    pub fn update_operation(&mut self, operation: &FuzzTargetOperation, is_ok: bool) {
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
    let Args {
        runs,
        run_len,
        threads,
    } = Args::parse();

    let path_to_read = Path::new("fuzz-report.txt");

    let available_parallelism = std::thread::available_parallelism().unwrap().get();

    assert!(
        threads <= available_parallelism,
        "Available parallelism: {:?}",
        available_parallelism
    );

    let successful_runs = Arc::new(Mutex::new(0));

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();

    let runs = (0..runs)
        .into_par_iter()
        .map(|_| generate_run(run_len))
        .collect::<Vec<_>>();

    fs::write(path_to_read, "\t\t\t✨ Fuzz report ✨\n\n").expect("unable to write");

    runs.par_iter().for_each(|operations| {
        let env = Env::default();
        let testing_env = TestingEnvironment::create(
            &env,
            TestingEnvConfig::default()
                .with_yaro_admin_deposit(410_000.0)
                .with_yusd_admin_deposit(440_000.0),
        );

        let mut logs = Vec::with_capacity(operations.len());
        let mut run_result = RunResult::default();

        for (operation_index, operation) in operations.iter().enumerate() {
            let result = operation.execute(&testing_env);
            let status = match result {
                Ok(_) => "✅",
                Err(_) => "❌",
            };

            logs.push(format!(
                "\t\t{status} {}. {}, result {:?}",
                operation_index + 1,
                &operation.to_string(),
                result
            ));

            run_result.update_operation(&operation, result.is_ok());

            testing_env.pool.assert_total_lp_less_or_equal_d();
        }

        let mut successful_runs = successful_runs.lock().unwrap();
        *successful_runs += 1;

        let log = format!(
            "✅ ({successful_runs} / {}) {}",
            runs.len(),
            &run_result.to_string()
        );

        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(path_to_read)
            .expect("unable to open file");
        let mut f = BufWriter::new(file);

        writeln!(
            f,
            "{}\n{}\n--------------------------------------------------------------------",
            log,
            logs.join("\n")
        )
        .expect("unable to write");

        print!("\r{log}");
        stdout().flush().unwrap();
    });
}
