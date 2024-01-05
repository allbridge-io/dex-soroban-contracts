use shared::{require, soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::Withdraw,
    storage::{pool::Pool, user_deposit::UserDeposit},
};

pub fn withdraw(env: Env, sender: Address, amount_lp: u128) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;

    require!(pool.can_withdraw, Error::Forbidden);

    let mut user_deposit = UserDeposit::get(&env, sender.clone());
    let (amount_a, amount_b) = pool.withdraw(&mut user_deposit, amount_lp)?;

    pool.get_token_a(&env).transfer(
        &env.current_contract_address(),
        &sender,
        &(amount_a as i128),
    );
    pool.get_token_b(&env).transfer(
        &env.current_contract_address(),
        &sender,
        &(amount_b as i128),
    );
    pool.get_lp_token(&env).burn(&sender, &(amount_lp as i128));

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    Withdraw {
        user: sender.clone(),
        amount: amount_lp,
    }
    .publish(&env);

    Ok(())
}
