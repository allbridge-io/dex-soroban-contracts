use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{token, Address, Env};
use storage::Admin;

use crate::common::Pool;

#[allow(clippy::too_many_arguments)]
pub fn initialize<const N: usize, P: Pool<N>>(
    env: Env,
    admin: Address,
    a: u128,
    tokens: [Address; N],
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<(), Error> {
    require!(!P::has(&env), Error::Initialized);

    require!(fee_share_bp < P::BP, Error::InvalidArg);
    require!(admin_fee_share_bp < P::BP, Error::InvalidArg);
    require!(a <= P::MAX_A, Error::InvalidArg);

    let mut decimals = [0; N];

    for (index, token) in tokens.iter().enumerate() {
        let decimal = token::Client::new(&env, &token).decimals();

        decimals[index] = decimal;
    }

    P::from_init_params(&env, a, tokens, decimals, fee_share_bp, admin_fee_share_bp).save(&env);
    Admin(admin).save(&env);

    Ok(())
}