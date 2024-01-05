use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{token, Address, Env};

use crate::storage::claimable_balance::ClaimableBalance;
use crate::{
    events::SwappedFromVUsd,
    storage::{bridge_address::Bridge, pool::Pool},
};

pub fn swap_from_v_usd(
    env: Env,
    user: Address,
    vusd_amount: u128,
    receive_amount_min: u128,
    zero_fee: bool,
    claimable: bool,
) -> Result<u128, Error> {
    let mut pool = Pool::get(&env)?;

    Bridge::require_exist_auth(&env)?;

    let (amount, fee) = pool.swap_b_to_a(vusd_amount, receive_amount_min, zero_fee)?;
    if claimable {
        ClaimableBalance::update(&env, user.clone(), |claimable_balance| {
            claimable_balance.amount += amount;
            Ok(())
        })?;
    } else {
        let token_client = token::Client::new(&env, &pool.token_a);
        token_client.transfer(&env.current_contract_address(), &user, &(amount as i128));
    }

    pool.save(&env);
    SwappedFromVUsd {
        token: pool.token_a,
        amount,
        vusd_amount,
        recipient: user,
        fee,
    }
    .publish(&env);

    Ok(amount)
}
