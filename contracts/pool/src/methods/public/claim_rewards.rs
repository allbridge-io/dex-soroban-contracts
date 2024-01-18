use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::RewardsClaimed,
    storage::{pool::Pool, user_deposit::UserDeposit},
};

pub fn claim_rewards(env: Env, sender: Address) -> Result<(), Error> {
    sender.require_auth();
    let pool = Pool::get(&env)?;

    let mut user_deposit = UserDeposit::get(&env, sender.clone());
    let rewards = pool.claim_rewards(&mut user_deposit)?;

    if rewards.to_array().into_iter().sum::<u128>() == 0 {
        return Ok(());
    }

    user_deposit.save(&env, sender.clone());

    for (index, reward) in rewards.to_array().into_iter().enumerate() {
        if reward == 0 {
            continue;
        }

        pool.get_token_by_index(&env, index).transfer(
            &env.current_contract_address(),
            &sender,
            &(reward as i128),
        );
    }

    RewardsClaimed {
        user: sender,
        rewards,
    }
    .publish(&env);

    Ok(())
}
