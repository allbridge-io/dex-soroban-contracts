use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};
use storage::Admin;

use crate::storage::user_deposit::UserDeposit;
use crate::storage::{common::Token, pool::Pool};

use super::internal::pool_view::WithdrawAmountView;

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
    Ok(Pool::get(&env)?.total_lp_amount)
}

pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
    Ok(UserDeposit::get(&env, user))
}

pub fn get_receive_amount(env: Env, input: u128, token_from: Token) -> Result<(u128, u128), Error> {
    let receive_amount = Pool::get(&env)?.get_receive_amount(input, token_from)?;
    Ok((receive_amount.output, receive_amount.fee))
}

pub fn get_send_amount(env: Env, output: u128, token_to: Token) -> Result<(u128, u128), Error> {
    Pool::get(&env)?.get_send_amount(output, token_to)
}

pub fn get_withdraw_amount(env: Env, lp_amount: u128) -> Result<WithdrawAmountView, Error> {
    Ok(Pool::get(&env)?.get_withdraw_amount(lp_amount)?.into())
}

pub fn get_deposit_amount(env: Env, amounts: (u128, u128)) -> Result<u128, Error> {
    let deposit_amount = Pool::get(&env)?.get_deposit_amount(amounts.into())?;

    Ok(deposit_amount.lp_amount)
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.0)
}
