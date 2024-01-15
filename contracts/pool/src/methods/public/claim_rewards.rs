use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::RewardsClaimed,
    storage::{
        pool::{Pool, Token},
        user_deposit::UserDeposit,
    },
};

pub fn claim_rewards(env: Env, sender: Address) -> Result<(), Error> {
    sender.require_auth();
    let pool = Pool::get(&env)?;

    let mut user_deposit = UserDeposit::get(&env, sender.clone());
    let rewards = pool.claim_rewards(&mut user_deposit)?;
    if rewards.0 + rewards.1 > 0 {
        user_deposit.save(&env, sender.clone());

        if rewards.0 > 0 {
            pool.get_token(&env, Token::A).transfer(
                &env.current_contract_address(),
                &sender,
                &(rewards.0 as i128),
            );
        }

        if rewards.1 > 0 {
            pool.get_token(&env, Token::B).transfer(
                &env.current_contract_address(),
                &sender,
                &(rewards.1 as i128),
            );
        }

        RewardsClaimed {
            user: sender,
            rewards,
        }
        .publish(&env);
    }

    Ok(())
}
