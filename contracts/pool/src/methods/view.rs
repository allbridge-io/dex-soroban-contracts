use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use crate::storage::bridge_address::Bridge;
use crate::storage::claimable_balance::ClaimableBalance;
use crate::storage::pool::Pool;
use crate::storage::user_deposit::UserDeposit;

pub fn pending_reward(env: Env, user: Address) -> Result<u128, Error> {
    let user_deposit = UserDeposit::get(&env, user);
    let pool = Pool::get(&env)?;
    Ok(
        ((user_deposit.lp_amount * pool.acc_reward_per_share_p) >> Pool::P)
            - user_deposit.reward_debt,
    )
}

pub fn get_pool(env: Env) -> Result<Pool, Error> {
    Pool::get(&env)
}

pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
    Ok(UserDeposit::get(&env, user))
}

pub fn get_bridge(env: Env) -> Result<Address, Error> {
    Ok(Bridge::get(&env)?.as_address())
}

pub fn get_claimable_balance(env: Env, address: Address) -> Result<u128, Error> {
    let claimable_balance = ClaimableBalance::get(&env, address);
    Ok(claimable_balance.amount)
}
