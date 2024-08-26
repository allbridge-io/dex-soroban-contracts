use shared::{require, Error, Event};
use soroban_sdk::{Address, Env, Vec};

use crate::{
    events::{Deposit, RewardsClaimed},
    pool::Pool,
    storage::{sized_array::SizedU128Array, user_deposit::UserDeposit},
};

pub fn deposit<const N: usize, P: Pool<N>>(
    env: Env,
    sender: Address,
    amounts: Vec<u128>,
    min_lp_amount: u128,
) -> Result<(), Error> {
    require!(amounts.len() as usize == N, Error::VecOutOfLimit);

    sender.require_auth();
    let mut pool = P::get(&env)?;
    let mut user_deposit = UserDeposit::get::<N>(&env, sender.clone());
    let amounts = SizedU128Array::from_vec(amounts);

    let (rewards, lp_amount) = pool.deposit(
        &env,
        amounts.clone(),
        sender.clone(),
        &mut user_deposit,
        min_lp_amount,
    )?;

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    Deposit {
        user: sender.clone(),
        lp_amount,
        amounts: amounts.get_inner(),
    }
    .publish(&env);

    if rewards.iter().sum::<u128>() != 0 {
        RewardsClaimed {
            user: sender,
            rewards: rewards.get_inner(),
        }
        .publish(&env);
    }

    Ok(())
}
