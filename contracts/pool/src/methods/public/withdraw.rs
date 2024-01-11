use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::Withdraw,
    storage::{pool::Pool, user_deposit::UserDeposit},
};

pub fn withdraw(env: Env, sender: Address, lp_amount: u128) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;

    let mut user_deposit = UserDeposit::get(&env, sender.clone());

    pool.withdraw(&env, sender.clone(), &mut user_deposit, lp_amount)?;

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    Withdraw {
        user: sender.clone(),
        lp_amount,
    }
    .publish(&env);

    Ok(())
}
