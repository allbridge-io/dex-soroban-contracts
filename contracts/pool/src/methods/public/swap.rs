use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::methods::internal::pool::Direction;
use crate::{events::Swapped, storage::pool::Pool};

pub fn swap(
    env: Env,
    sender: Address,
    recipient: Address,
    amount_in: u128,
    receive_amount_min: u128,
    zero_fee: bool,
    direction: Direction,
) -> Result<u128, Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;

    let (amount, fee) = pool.swap(
        &env,
        sender.clone(),
        recipient.clone(),
        amount_in,
        receive_amount_min,
        zero_fee,
        direction,
    )?;

    pool.save(&env);

    Swapped {
        from_token: pool.tokens.get_unchecked(0),
        to_token: pool.tokens.get_unchecked(1),
        from_amount: amount_in,
        to_amount: amount,
        sender,
        recipient,
        fee,
    }
    .publish(&env);

    Ok(amount)
}
