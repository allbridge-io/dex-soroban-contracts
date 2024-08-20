use shared::{Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    common::Pool,
    events::{DepositEvent, RewardsClaimedEvent},
    storage::{sized_array::SizedU128Array, user_deposit::UserDeposit},
};

pub fn deposit<const N: usize, P: Pool<N>>(
    env: Env,
    sender: Address,
    amounts: (u128, u128, u128),
    min_lp_amount: u128,
) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = P::get(&env)?;
    let mut user_deposit = UserDeposit::get::<N>(&env, sender.clone());
    let amounts = SizedU128Array::from_array(&env, [amounts.0, amounts.1, amounts.2]);

    let (rewards, lp_amount) = pool.deposit(
        &env,
        amounts.clone(),
        sender.clone(),
        &mut user_deposit,
        min_lp_amount,
    )?;

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    P::Deposit::create(sender.clone(), lp_amount, amounts).publish(&env);

    if rewards.iter().sum::<u128>() != 0 {
        P::RewardsClaimed::create(sender, rewards).publish(&env);
    }

    Ok(())
}
