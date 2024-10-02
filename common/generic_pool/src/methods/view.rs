use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env, Vec};
use storage::Admin;

use crate::{
    pool::{Pool, WithdrawAmountView},
    storage::{sized_array::SizedU128Array, user_deposit::UserDeposit},
};

pub fn pending_reward<const N: usize, P: Pool<N>>(
    env: Env,
    user: Address,
) -> Result<soroban_sdk::Vec<u128>, Error> {
    let user = UserDeposit::get::<N>(&env, user);
    let pool = P::get(&env)?;

    Ok(pool.get_pending(&env, &user).get_inner())
}

pub fn get_pool<const N: usize, P: Pool<N>>(env: Env) -> Result<P, Error> {
    P::get(&env)
}

pub fn get_d<const N: usize, P: Pool<N>>(env: Env) -> Result<u128, Error> {
    Ok(P::get(&env)?.total_lp_amount())
}

pub fn get_user_deposit<const N: usize>(env: Env, user: Address) -> Result<UserDeposit, Error> {
    Ok(UserDeposit::get::<N>(&env, user))
}

pub fn get_receive_amount<const N: usize, P: Pool<N>>(
    env: Env,
    input: u128,
    token_from: P::Token,
    token_to: P::Token,
) -> Result<Vec<u128>, Error> {
    let receive_amount = P::get(&env)?.get_receive_amount(input, token_from, token_to)?;

    Ok(Vec::from_array(
        &env,
        [receive_amount.output, receive_amount.fee],
    ))
}

pub fn get_send_amount<const N: usize, P: Pool<N>>(
    env: Env,
    output: u128,
    token_from: P::Token,
    token_to: P::Token,
) -> Result<Vec<u128>, Error> {
    P::get(&env)?
        .get_send_amount(output, token_from, token_to)
        .map(|(v1, v2)| Vec::from_array(&env, [v1, v2]))
}

pub fn get_withdraw_amount<const N: usize, P: Pool<N>>(
    env: Env,
    lp_amount: u128,
) -> Result<WithdrawAmountView, Error> {
    Ok(P::get(&env)?.get_withdraw_amount(&env, lp_amount)?.into())
}

pub fn get_deposit_amount<const N: usize, P: Pool<N>>(
    env: Env,
    amounts: Vec<u128>,
) -> Result<u128, Error> {
    require!(amounts.len() as usize == N, Error::UnexpectedVecSize);
    let deposit_amount =
        P::get(&env)?.get_deposit_amount(&env, SizedU128Array::from_vec(amounts))?;

    Ok(deposit_amount.lp_amount)
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.0)
}
