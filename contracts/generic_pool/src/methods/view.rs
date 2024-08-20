use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};
use storage::Admin;

use crate::{
    common::{Pool, WithdrawAmount},
    storage::{sized_array::SizedU128Array, user_deposit::UserDeposit},
};

pub fn pending_reward<const N: usize, P: Pool<N>>(
    env: Env,
    user: Address,
) -> Result<(u128, u128), Error> {
    let user = UserDeposit::get::<N>(&env, user);
    let pool = P::get(&env)?;

    let pending = pool.get_pending(&env, &user);

    Ok((pending.get(0usize), pending.get(1usize)))
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
) -> Result<(u128, u128), Error> {
    let receive_amount = P::get(&env)?.get_receive_amount(input, token_from, token_to)?;
    Ok((receive_amount.output, receive_amount.fee))
}

pub fn get_send_amount<const N: usize, P: Pool<N>>(
    env: Env,
    output: u128,
    token_from: P::Token,
    token_to: P::Token,
) -> Result<(u128, u128), Error> {
    P::get(&env)?.get_send_amount(output, token_from, token_to)
}

pub fn get_withdraw_amount<const N: usize, P: Pool<N>, WA: From<WithdrawAmount<N>>>(
    env: Env,
    lp_amount: u128,
) -> Result<WA, Error> {
    Ok(P::get(&env)?.get_withdraw_amount(&env, lp_amount)?.into())
}

pub fn get_deposit_amount<const N: usize, P: Pool<N>>(
    env: Env,
    amounts: (u128, u128, u128),
) -> Result<u128, Error> {
    let deposit_amount = P::get(&env)?.get_deposit_amount(
        &env,
        SizedU128Array::from_array(&env, [amounts.0, amounts.1, amounts.2]),
    )?;

    Ok(deposit_amount.lp_amount)
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.0)
}
