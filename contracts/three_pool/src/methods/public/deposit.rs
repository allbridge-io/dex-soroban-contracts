use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::{Deposit, RewardsClaimed},
    storage::{pool::Pool, triple_values::TripleU128, user_deposit::UserDeposit},
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
    let amounts = TripleU128::from(amounts);

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
        amounts: amounts.data,
    }
    .publish(&env);

    if !rewards.is_zero() {
        RewardsClaimed {
            user: sender,
            rewards: rewards.data,
        }
        .publish(&env);
    }

    Ok(())
}
