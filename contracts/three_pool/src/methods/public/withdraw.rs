use shared::{Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    common::Pool,
    events::{RewardsClaimedEvent, WithdrawEvent},
    storage::user_deposit::UserDeposit,
};

pub fn withdraw<const N: usize, P: Pool<N>>(
    env: Env,
    sender: Address,
    lp_amount: u128,
) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = P::get(&env)?;
    let mut user_deposit = UserDeposit::get::<N>(&env, sender.clone());

    let (withdraw_amount, rewards) =
        pool.withdraw(&env, sender.clone(), &mut user_deposit, lp_amount)?;

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    P::Withdraw::create(
        sender.clone(),
        lp_amount,
        withdraw_amount.amounts,
        withdraw_amount.fees,
    )
    .publish(&env);

    if rewards.iter().sum::<u128>() != 0 {
        P::RewardsClaimed::create(sender, rewards).publish(&env);
    }

    Ok(())
}
