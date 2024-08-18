use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::{Deposit, RewardsClaimed},
    storage::{pool::Pool, sized_array::SizedU128Array, user_deposit::UserDeposit},
};

pub fn deposit(
    env: Env,
    sender: Address,
    amounts: (u128, u128, u128),
    min_lp_amount: u128,
) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;
    let mut user_deposit = UserDeposit::get(&env, sender.clone());
    let amounts = SizedU128Array::from_array(&env, [amounts.0, amounts.1, amounts.2]);

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
        amounts: (amounts.get(0), amounts.get(1), amounts.get(2)),
    }
    .publish(&env);

    if !rewards.iter().sum::<u128>() == 0 {
        RewardsClaimed {
            user: sender,
            rewards: (rewards.get(0), rewards.get(1), rewards.get(2)),
        }
        .publish(&env);
    }

    Ok(())
}
