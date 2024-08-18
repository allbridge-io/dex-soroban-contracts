use shared::{Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::RewardsClaimed, methods::internal::pool::Pool, storage::user_deposit::UserDeposit,
};

pub fn claim_rewards<P: Pool>(env: Env, sender: Address) -> Result<(), Error> {
    sender.require_auth();
    let pool = P::get(&env)?;

    let mut user_deposit = UserDeposit::get(&env, sender.clone());
    let rewards = pool.claim_rewards(&env, sender.clone(), &mut user_deposit)?;

    if rewards.iter().sum::<u128>() == 0 {
        return Ok(());
    }

    user_deposit.save(&env, sender.clone());

    RewardsClaimed {
        user: sender,
        rewards: (rewards.get(0), rewards.get(1), rewards.get(2)),
    }
    .publish(&env);

    Ok(())
}
