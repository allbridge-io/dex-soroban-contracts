use shared::{soroban_data::SimpleSorobanData, Error, Event};
use soroban_sdk::{token, Address, Env};

use crate::events::BalanceClaimed;
use crate::storage::claimable_balance::ClaimableBalance;
use crate::storage::pool::Pool;

pub fn claim_balance(env: Env, user: Address) -> Result<(), Error> {
    let pool = Pool::get(&env)?;
    let mut claimable_balance = ClaimableBalance::get(&env, user.clone());

    if claimable_balance.amount > 0 {
        let token_client = token::Client::new(&env, &pool.token_a);

        let amount = claimable_balance.amount;
        claimable_balance.amount = 0;
        claimable_balance.save(&env, user.clone());

        token_client.transfer(&env.current_contract_address(), &user, &(amount as i128));

        BalanceClaimed { user, amount }.publish(&env);
    }

    Ok(())
}
