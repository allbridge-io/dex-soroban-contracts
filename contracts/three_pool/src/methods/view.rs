use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};
use storage::Admin;

use crate::storage::sized_array::SizedU128Array;
use crate::storage::user_deposit::UserDeposit;
use crate::storage::{common::Token, pool::ThreePool};

use super::internal::pool::Pool;
use super::internal::pool_view::WithdrawAmountView;

pub fn pending_reward(env: Env, user: Address) -> Result<(u128, u128), Error> {
    let user = UserDeposit::get(&env, user);
    let pool = ThreePool::get(&env)?;

    let pending = pool.get_pending(&env, &user);

    Ok((pending.get(0), pending.get(1)))
}

pub fn get_pool(env: Env) -> Result<ThreePool, Error> {
    ThreePool::get(&env)
}

pub fn get_d(env: Env) -> Result<u128, Error> {
    Ok(ThreePool::get(&env)?.total_lp_amount)
}

pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
    Ok(UserDeposit::get(&env, user))
}

pub fn get_receive_amount(
    env: Env,
    input: u128,
    token_from: Token,
    token_to: Token,
) -> Result<(u128, u128), Error> {
    let receive_amount = ThreePool::get(&env)?.get_receive_amount(input, token_from, token_to)?;
    Ok((receive_amount.output, receive_amount.fee))
}

pub fn get_send_amount(
    env: Env,
    output: u128,
    token_from: Token,
    token_to: Token,
) -> Result<(u128, u128), Error> {
    ThreePool::get(&env)?.get_send_amount(output, token_from, token_to)
}

pub fn get_withdraw_amount(env: Env, lp_amount: u128) -> Result<WithdrawAmountView, Error> {
    Ok(ThreePool::get(&env)?
        .get_withdraw_amount(&env, lp_amount)?
        .into())
}

pub fn get_deposit_amount(env: Env, amounts: (u128, u128, u128)) -> Result<u128, Error> {
    let deposit_amount = ThreePool::get(&env)?.get_deposit_amount(
        &env,
        SizedU128Array::from_array(&env, [amounts.0, amounts.1, amounts.2]),
    )?;

    Ok(deposit_amount.lp_amount)
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.0)
}
