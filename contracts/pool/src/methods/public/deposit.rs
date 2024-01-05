use shared::{require, soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::{Deposit, RewardsClaimed},
    methods::internal::pool::DepositResult,
    storage::{pool::Pool, user_deposit::UserDeposit},
};

pub fn deposit(env: Env, sender: Address, amount_sp: u128) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = Pool::get(&env)?;

    require!(pool.can_deposit, Error::Forbidden);

    let mut user_deposit = UserDeposit::get(&env, sender.clone());

    let DepositResult {
        rewards,
        lp_amount,
        token_a_amount,
        token_b_amount,
    } = pool.deposit(amount_sp, &mut user_deposit)?;

    pool.get_token_a(&env).transfer(
        &sender,
        &env.current_contract_address(),
        &(token_a_amount as i128),
    );
    pool.get_token_b(&env).transfer(
        &sender,
        &env.current_contract_address(),
        &(token_b_amount as i128),
    );
    pool.get_lp_native_asset(&env)
        .mint(&sender, &(lp_amount as i128));

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
