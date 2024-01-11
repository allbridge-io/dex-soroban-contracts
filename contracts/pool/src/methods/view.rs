use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use crate::storage::pool::Pool;
use crate::storage::user_deposit::UserDeposit;

pub fn pending_reward(env: Env, user: Address) -> Result<u128, Error> {
    let _user_deposit = UserDeposit::get(&env, user);
    let _pool = Pool::get(&env)?;

    todo!()
}

pub fn get_pool(env: Env) -> Result<Pool, Error> {
    Pool::get(&env)
}

pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
    Ok(UserDeposit::get(&env, user))
}
