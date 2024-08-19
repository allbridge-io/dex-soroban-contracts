use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{token, Address, Env};
use storage::Admin;

use crate::storage::pool::Pool;

#[allow(clippy::too_many_arguments)]
pub fn initialize(
    env: Env,
    admin: Address,
    a: u128,
    tokens: [Address; 2],
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<(), Error> {
    require!(!Pool::has(&env), Error::Initialized);

    require!(fee_share_bp < Pool::BP, Error::InvalidArg);
    require!(admin_fee_share_bp < Pool::BP, Error::InvalidArg);
    require!(a <= Pool::MAX_A, Error::InvalidArg);

    let mut decimals = [0; 2];

    for (index, token) in tokens.iter().enumerate() {
        let decimal = token::Client::new(&env, &token).decimals();

        decimals[index] = decimal;
    }

    Pool::from_init_params(a, tokens, decimals, fee_share_bp, admin_fee_share_bp).save(&env);
    Admin(admin).save(&env);

    Ok(())
}
