use shared::{Error, Event};
use soroban_sdk::{Address, Env};

use crate::{events::RewardsClaimed, pool::Pool, storage::user_deposit::UserDeposit};

pub fn claim_rewards<const N: usize, P: Pool<N>>(env: Env, sender: Address) -> Result<(), Error> {
    sender.require_auth();
    let pool = P::get(&env)?;

    let mut user_deposit = UserDeposit::get::<N>(&env, sender.clone());
    let rewards = pool.claim_rewards(&env, sender.clone(), &mut user_deposit)?;

    if rewards.iter().sum::<u128>() == 0 {
        return Ok(());
    }

    user_deposit.save(&env, sender.clone());

    RewardsClaimed {
        user: sender,
        rewards: rewards.get_inner(),
    }
    .publish(&env);

    Ok(())
}
