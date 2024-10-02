use shared::{Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    pool::Pool,
    events::{RewardsClaimed, Withdraw},
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

    Withdraw {
        user: sender.clone(),
        lp_amount,
        amounts: withdraw_amount.amounts.get_inner(),
        fees: withdraw_amount.fees.get_inner(),
    }
    .publish(&env);

    if rewards.iter().sum::<u128>() != 0 {
        RewardsClaimed {
            user: sender,
            rewards: rewards.get_inner(),
        }
        .publish(&env);
    }

    Ok(())
}
