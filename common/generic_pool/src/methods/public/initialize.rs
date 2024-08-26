use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{token, Address, Env, Vec};
use storage::Admin;

use crate::pool::Pool;

pub fn initialize<const N: usize, P: Pool<N>>(
    env: Env,
    admin: Address,
    a: u128,
    tokens: Vec<Address>,
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<(), Error> {
    require!(tokens.len() as usize == N, Error::UnexpectedVecSize);

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
