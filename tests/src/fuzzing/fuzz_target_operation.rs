use rand::distributions::{Distribution, Standard};
use rand::Rng;
use rand_derive2::RandGen;
use tabled::Tabled;

use crate::contracts::pool::Direction;
use crate::utils::{CallResult, TestingEnvironment, User};

#[derive(Debug, Clone, Default, Tabled)]
pub struct Action {
    pub status: &'static str,
    pub index: usize,
    pub log: String,
    pub d: u128,
    pub total_lp: u128,
    pub invariant: String,
}

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Amount(pub f64);

impl Distribution<Amount> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Amount {
        let v1 = rng.gen::<u16>() as f64;
        let v2 = rng.gen::<u16>() as f64;

        Amount(v1 + v2)
    }
}

#[derive(Debug, RandGen)]
pub enum FuzzTargetOperation {
    Swap {
        direction: SwapDirection,
        amount: Amount,
        sender: UserID,
        recipient: UserID,
    },
    Withdraw {
        lp_amount: Amount,
        user: UserID,
    },
    Deposit {
        yusd_amount: Amount,
        yaro_amount: Amount,
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
                    "**[Swap]** {} {:?}, from *{:?}* to *{:?}*",
                    amount.0, direction, sender, recipient
                )
            }

            FuzzTargetOperation::Deposit {
                yaro_amount,
                yusd_amount,
                user,
            } => {
                format!(
                    "**[Deposit]** *{:?}*, amounts: {} Yaro {} Yusd",
                    user, yaro_amount.0, yusd_amount.0,
                )
            }

            FuzzTargetOperation::Withdraw { lp_amount, user } => {
                format!("**[Withdraw]** *{:?}*, lp amount: {}", user, lp_amount.0)
            }
        }
    }
}

impl FuzzTargetOperation {
    pub fn generate_run(len: usize) -> Vec<FuzzTargetOperation> {
        let mut rng = rand::thread_rng();

        (0..len).into_iter().map(|_| rng.gen()).collect()
    }

    fn get_user(user_id: UserID, testing_env: &TestingEnvironment) -> &User {
        match user_id {
            UserID::Alice => &testing_env.alice,
            UserID::Bob => &testing_env.bob,
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
                let direction: Direction = (*direction).into();
                let (token_from, _) = testing_env.get_tokens_by_direction(direction.clone());

                if token_from.balance_of_f64(sender.as_ref()) - amount.0 <= 0.0 {
                    return Ok(());
                }

                testing_env
                    .pool
                    .swap(sender, recipient, amount.0, 0.0, direction)
                    .map(|_| ())
            }

            FuzzTargetOperation::Deposit {
                yaro_amount,
                yusd_amount,
                user,
            } => {
                let sender = Self::get_user(*user, testing_env);
                if yusd_amount.0 + yaro_amount.0 == 0.0 {
                    return Ok(());
                }

                testing_env
                    .pool
                    .deposit(sender, (yusd_amount.0, yaro_amount.0), 0.0)
            }

            FuzzTargetOperation::Withdraw { lp_amount, user } => {
                let sender = Self::get_user(*user, testing_env);
                testing_env.pool.withdraw(sender, lp_amount.0)
            }
        }
    }

    pub fn get_log_string(&self, result: &CallResult) -> String {
        let log = format!("{}", &self.to_string());
        match result {
            Ok(_) => log,
            Err(err) => format!("{}, error: {:?}", log.as_str(), err),
        }
    }
}
