use shared::{require, soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::{Deposit, RewardsClaimed},
    storage::{pool::Pool, user_deposit::UserDeposit},
};

pub fn deposit(env: Env, sender: Address, amount_sp: u128) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;

    require!(pool.can_deposit, Error::Forbidden);

    let mut user_deposit = UserDeposit::get(&env, sender.clone());

    let (rewards, lp_amount) = pool.deposit(&env, amount_sp, sender.clone(), &mut user_deposit)?;

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    Deposit {
        user: sender.clone(),
        amount: lp_amount,
    }
    .publish(&env);

    RewardsClaimed {
        user: sender.clone(),
        amount: rewards,
    }
    .publish(&env);

    Ok(())
}
