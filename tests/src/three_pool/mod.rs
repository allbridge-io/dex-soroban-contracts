pub mod admin;
pub mod claims;
pub mod deposit;
pub mod swap;
pub mod withdraw;

mod three_pool_snapshot;
mod three_pool_testing_env;

pub use three_pool_snapshot::*;
pub use three_pool_testing_env::*;

#[cfg(test)]
pub struct DepositArgs {
    amounts: (f64, f64, f64),
    min_lp: f64,
}

#[cfg(test)]
pub struct DoWithdrawArgs {
    amount: f64,
    expected_amounts: (f64, f64, f64),
    expected_fee: (f64, f64, f64),
    expected_rewards: (f64, f64, f64),
    expected_user_lp_diff: f64,
    expected_admin_fee: (f64, f64, f64),
}
