use std::sync::{Arc, Mutex};

use clap::Parser;
use clap_derive::Parser;
use rand::Rng;
use rand_derive2::RandGen;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use soroban_sdk::testutils::arbitrary::{arbitrary, fuzz_catch_panic, Arbitrary};
use soroban_sdk::Env;

use tests::contracts::pool::Direction;
use tests::utils::{CallResult, TestingEnvConfig, TestingEnvironment, Token, User};

#[derive(Arbitrary, Debug, Clone, Copy, RandGen)]
pub enum SwapDirection {
    A2B,
    B2A,
}

impl Into<Direction> for SwapDirection {
    fn into(self) -> Direction {
        match self {
            SwapDirection::A2B => Direction::A2B,
            SwapDirection::B2A => Direction::B2A,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, RandGen)]
pub enum UserID {
    Alice,
    Bob,
}

#[derive(Debug, RandGen)]
pub struct Swap {
    direction: SwapDirection,
    amount: u16,
    sender: UserID,
    recipient: UserID,
}

#[derive(Debug, RandGen)]
pub struct Withdraw {
    lp_amount: u16,
    user: UserID,
}

#[derive(Debug, RandGen)]
pub struct Deposit {
    yusd_amount: u16,
    yaro_amount: u16,
    user: UserID,
}

#[derive(Debug, RandGen)]
pub enum FuzzTargetOperation {
    Swap(Swap),
    Withdraw(Withdraw),
    Deposit(Deposit),
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
            FuzzTargetOperation::Swap(swap) => {
                let Swap {
                    direction,
                    amount,
                    sender,
                    recipient,
                } = swap;

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

            FuzzTargetOperation::Deposit(deposit) => {
                let Deposit {
                    yaro_amount,
                    yusd_amount,
                    user,
                } = deposit;

                let sender = Self::get_user(*user, testing_env);
                let deposits = (*yusd_amount as f64, *yaro_amount as f64);
                if deposits.0 + deposits.1 == 0.0 {
                    return Ok(());
                }

                testing_env.pool.deposit(sender, deposits, 0.0)
            }

            FuzzTargetOperation::Withdraw(withdraw) => {
                let Withdraw { lp_amount, user } = withdraw;
                let sender = Self::get_user(*user, testing_env);
                let lp_amount = (*lp_amount) as f64;

                testing_env.pool.withdraw(sender, lp_amount)
            }
        }
    }
}

pub fn generate_run(len: usize) -> Vec<FuzzTargetOperation> {
    let mut rng = rand::thread_rng();

    (0..len)
        .into_iter()
        .map(|_| rng.gen())
        .collect::<Vec<FuzzTargetOperation>>()
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

fn main() {
    let Args {
        runs,
        run_len,
        threads,
    } = Args::parse();

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

    runs.par_iter().for_each(|operations| {
        let env = Env::default();
        let testing_env = TestingEnvironment::create(
            &env,
            TestingEnvConfig::default()
                .with_yaro_admin_deposit(410_000.0)
                .with_yusd_admin_deposit(440_000.0),
        );

        for operation in operations {
            let _ = fuzz_catch_panic(|| operation.execute(&testing_env));

            testing_env.pool.assert_total_lp_less_or_equal_d();
        }

        let mut successful_runs = successful_runs.lock().unwrap();
        *successful_runs += 1;

        println!("✅ {successful_runs} / {}", runs.len());
    });
}
