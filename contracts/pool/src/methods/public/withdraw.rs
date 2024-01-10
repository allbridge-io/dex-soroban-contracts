use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::Withdraw,
    storage::{pool::Pool, user_deposit::UserDeposit},
};

pub fn withdraw(env: Env, sender: Address, amount_lp: u128) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;

    let mut user_deposit = UserDeposit::get(&env, sender.clone());

    pool.withdraw(&env, sender.clone(), &mut user_deposit, amount_lp)?;

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    Withdraw {
        user: sender.clone(),
        amount: amount_lp,
    }
    .publish(&env);

    Ok(())
}
