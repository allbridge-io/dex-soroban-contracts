use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::Swapped,
    storage::{common::Direction, pool::Pool},
};

pub fn swap(
    env: Env,
    sender: Address,
    recipient: Address,
    from_amount: u128,
    receive_amount_min: u128,
    direction: Direction,
) -> Result<u128, Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;

    let (to_amount, fee) = pool.swap(
        &env,
        sender.clone(),
        recipient.clone(),
        from_amount,
        receive_amount_min,
        direction,
    )?;

    pool.save(&env);

    let (token_from, token_to) = direction.get_tokens();

    Swapped {
        from_token: pool.tokens[token_from].clone(),
        to_token: pool.tokens[token_to].clone(),
        from_amount,
        to_amount,
        sender,
        recipient,
        fee,
    }
    .publish(&env);

    Ok(to_amount)
}
