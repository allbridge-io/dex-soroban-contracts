use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::SwappedToVUsd,
    storage::{bridge_address::Bridge, pool::Pool},
};

pub fn swap_to_v_usd(env: Env, user: Address, amount: u128, zero_fee: bool) -> Result<u128, Error> {
    user.require_auth();
    let mut pool = Pool::get(&env)?;

    Bridge::require_exist_auth(&env)?;

    let current_pool = env.current_contract_address();
    let token_client = soroban_sdk::token::Client::new(&env, &pool.token_a);

    token_client.transfer(&user, &current_pool, &(amount as i128));

    let (vusd_amount, fee) = pool.swap_to_token_b(amount, zero_fee)?;
    pool.save(&env);

    SwappedToVUsd {
        token: pool.token_a,
        amount,
        vusd_amount,
        sender: user,
        fee,
    }
    .publish(&env);

    Ok(vusd_amount)
}
