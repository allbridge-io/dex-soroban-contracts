use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use crate::storage::pool::Pool;
use crate::storage::user_deposit::UserDeposit;

pub fn pending_reward(env: Env, user: Address) -> Result<(u128, u128), Error> {
    let user = UserDeposit::get(&env, user);
    let pool = Pool::get(&env)?;

    let pending = pool.get_pending(&user);

    Ok((pending[0], pending[1]))
}

pub fn get_pool(env: Env) -> Result<Pool, Error> {
    Pool::get(&env)
}

pub fn get_d(env: Env) -> Result<u128, Error> {
    Ok(Pool::get(&env)?.get_current_d())
}

pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
    Ok(UserDeposit::get(&env, user))
}
