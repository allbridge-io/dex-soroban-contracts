use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{token, Address, Env};
use storage::Admin;

use crate::storage::pool::Pool;

#[allow(clippy::too_many_arguments)]
pub fn initialize(
    env: Env,
    admin: Address,
    a: u128,
    token_a: Address,
    token_b: Address,
    token_c: Address,
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<(), Error> {
    require!(!Pool::has(&env), Error::Initialized);

    require!(fee_share_bp < Pool::BP, Error::InvalidArg);
    require!(admin_fee_share_bp < Pool::BP, Error::InvalidArg);
    require!(a <= Pool::MAX_A, Error::InvalidArg);

    let decimals_a = token::Client::new(&env, &token_a).decimals();
    let decimals_b = token::Client::new(&env, &token_b).decimals();
    let decimals_c = token::Client::new(&env, &token_c).decimals();

    Pool::from_init_params(
        a,
        token_a,
        token_b,
        token_c,
        (decimals_a, decimals_b, decimals_c),
        fee_share_bp,
        admin_fee_share_bp,
    )
    .save(&env);
    Admin(admin).save(&env);

    Ok(())
}
