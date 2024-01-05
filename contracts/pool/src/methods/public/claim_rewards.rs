use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{token, Address, Env};

use crate::{
    events::RewardsClaimed,
    storage::{pool::Pool, user_deposit::UserDeposit},
};

pub fn claim_rewards(env: Env, sender: Address) -> Result<(), Error> {
    sender.require_auth();
    let pool = Pool::get(&env)?;

    let mut user_deposit = UserDeposit::get(&env, sender.clone());
    let amount = pool.claim_rewards(&mut user_deposit)?;
    if amount > 0 {
        user_deposit.save(&env, sender.clone());
        let token_client = token::Client::new(&env, &pool.token_a);

        token_client.transfer(&env.current_contract_address(), &sender, &(amount as i128));

        RewardsClaimed {
            user: sender,
            amount,
        }
        .publish(&env);
    }

    Ok(())
}
