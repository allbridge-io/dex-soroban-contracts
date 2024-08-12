use std::fmt::Display;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use rand_derive2::RandGen;
use serde_derive::Serialize;

use crate::three_pool_utils::{CallResult, TestingEnv, Token, User};

#[derive(Debug, Clone, Default)]
pub struct Action {
    pub status: &'static str,
    pub index: usize,
    pub log: String,
    pub d: u128,
    pub total_lp: u128,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ActionPoolChange {
    pub d: u128,
    pub total_lp: u128,
    pub diff: i128,
}

impl From<Action> for ActionPoolChange {
    fn from(Action { d, total_lp, .. }: Action) -> Self {
        Self {
            d,
            total_lp,
            diff: total_lp as i128 - d as i128,
        }
    }
}

#[derive(Debug, Clone, Copy, RandGen)]
pub enum SwapDirection {
    A2B,
    A2C,
    B2A,
    B2C,
    C2A,
    C2B
}

impl SwapDirection {
    pub fn get_token_pair<'a>(&self, testing_env: &'a TestingEnv) -> (&'a Token, &'a Token) {
        match self {
            SwapDirection::A2B => (&testing_env.token_a, &testing_env.token_b),
            SwapDirection::A2C => (&testing_env.token_a, &testing_env.token_c),
            SwapDirection::B2A => (&testing_env.token_b, &testing_env.token_a),
            SwapDirection::B2C => (&testing_env.token_b, &testing_env.token_c),
            SwapDirection::C2A => (&testing_env.token_c, &testing_env.token_a),
            SwapDirection::C2B => (&testing_env.token_c, &testing_env.token_b),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, RandGen)]
pub enum UserID {
    Alice,
    Bob,
}

impl UserID {
    pub fn get_user<'a>(&self, testing_env: &'a TestingEnv) -> &'a User {
        match self {
            UserID::Alice => &testing_env.alice,
            UserID::Bob => &testing_env.bob,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Amount(pub f64);

impl Distribution<Amount> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Amount {
        Amount(rng.gen_range(1..100_000) as f64)
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
        a_amount: Amount,
        b_amount: Amount,
        c_amount: Amount,
        user: UserID,
    },
}

impl Display for FuzzTargetOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
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
                a_amount,
                b_amount,
                c_amount,
                user,
            } => {
                format!(
                    "**[Deposit]** *{:?}*, amounts: {} A {} B {} C",
                    user, a_amount.0, b_amount.0, c_amount.0
                )
            }

            FuzzTargetOperation::Withdraw { lp_amount, user } => {
                format!("**[Withdraw]** *{:?}*, lp amount: {}", user, lp_amount.0)
            }
        };
        write!(f, "{}", str)
    }
}

impl FuzzTargetOperation {
    pub fn generate_run(len: usize) -> Vec<FuzzTargetOperation> {
        let mut rng = rand::thread_rng();

        (&mut rng).sample_iter(Standard).take(len).collect()
    }

    pub fn execute(&self, testing_env: &TestingEnv) -> CallResult {
        match self {
            FuzzTargetOperation::Swap {
                direction,
                amount,
                sender,
                recipient,
            } => {
                let sender = sender.get_user(testing_env);
                let recipient = recipient.get_user(testing_env);
                let (token_from, token_to) = direction.get_token_pair(testing_env);
                testing_env
                    .pool
                    .swap_checked(sender, recipient, amount.0, 0.0, token_from,  token_to)?;

                Ok(())
            }

            FuzzTargetOperation::Deposit {
                b_amount,
                a_amount,
                c_amount,
                user,
            } => testing_env.pool.deposit_checked(
                user.get_user(testing_env),
                (a_amount.0, b_amount.0, c_amount.0),
                0.0,
            ),

            FuzzTargetOperation::Withdraw { lp_amount, user } => testing_env
                .pool
                .withdraw_checked(user.get_user(testing_env), lp_amount.0),
        }
    }

    pub fn get_log_string(&self, result: &CallResult) -> String {
        let log = self.to_string();
        match result {
            Ok(_) => log,
            Err(err) => format!("{}, error: {:?}", log.as_str(), err),
        }
    }
}
