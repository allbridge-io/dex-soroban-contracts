use shared::{soroban_data::SimpleSorobanData, utils::safe_cast, Error};
use soroban_sdk::Env;
use storage::Admin;

use crate::methods::internal::pool::Pool;

pub fn claim_admin_fee<P: Pool>(env: Env) -> Result<(), Error> {
    let admin = Admin::get(&env)?;
    admin.require_auth();

    let mut pool = P::get(&env)?;

    for (index, _) in pool.tokens().iter().enumerate() {
        if pool.admin_fee_amount().get(index) > 0 {
            pool.get_token_by_index(&env, index).transfer(
                &env.current_contract_address(),
                admin.as_ref(),
                &safe_cast(pool.admin_fee_amount().get(index))?,
            );
            pool.admin_fee_amount_mut().set(index, 0);
            pool.save(&env);
        }
    }

    Ok(())
}
