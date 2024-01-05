use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{token, Env};

use crate::storage::{admin::Admin, pool::Pool};

pub fn claim_admin_fee(env: Env) -> Result<(), Error> {
    let admin = Admin::get(&env)?;
    admin.require_auth();

    let mut pool = Pool::get(&env)?;

    if pool.admin_fee_amount > 0 {
        let token_client = token::Client::new(&env, &pool.token_a);

        token_client.transfer(
            &env.current_contract_address(),
            admin.as_ref(),
            &(pool.admin_fee_amount as i128),
        );
        pool.admin_fee_amount = 0;
        pool.save(&env);
    }

    Ok(())
}
