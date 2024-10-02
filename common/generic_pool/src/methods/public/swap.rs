use shared::{Error, Event};
use soroban_sdk::{Address, Env};

use crate::{pool::Pool, events::Swapped};

pub fn swap<const N: usize, P: Pool<N>>(
    env: Env,
    sender: Address,
    recipient: Address,
    from_amount: u128,
    receive_amount_min: u128,
    token_from: P::Token,
    token_to: P::Token,
) -> Result<u128, Error> {
    sender.require_auth();
    let mut pool = P::get(&env)?;

    let (to_amount, fee) = pool.swap(
        &env,
        sender.clone(),
        recipient.clone(),
        from_amount,
        receive_amount_min,
        token_from,
        token_to,
    )?;

    pool.save(&env);

    Swapped {
        from_token: pool.tokens().get(token_from),
        to_token: pool.tokens().get(token_to),
        from_amount,
        to_amount,
        sender,
        recipient,
        fee,
    }
    .publish(&env);

    Ok(to_amount)
}
