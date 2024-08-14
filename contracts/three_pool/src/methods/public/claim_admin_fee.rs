use shared::{soroban_data::SimpleSorobanData, utils::safe_cast, Error};
use soroban_sdk::Env;
use storage::Admin;

use crate::storage::pool::Pool;

pub fn claim_admin_fee(env: Env) -> Result<(), Error> {
    let admin = Admin::get(&env)?;
    admin.require_auth();

    let mut pool = Pool::get(&env)?;

    for (index, _) in pool.tokens.to_array().into_iter().enumerate() {
        if pool.admin_fee_amount[index] > 0 {
            pool.get_token_by_index(&env, index).transfer(
                &env.current_contract_address(),
                admin.as_ref(),
                &safe_cast(pool.admin_fee_amount[index])?,
            );
            pool.admin_fee_amount[index] = 0;
            pool.save(&env);
        }
    }

    Ok(())
}
