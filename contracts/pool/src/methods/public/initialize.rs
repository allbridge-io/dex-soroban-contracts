use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use crate::storage::pool::Pool;

#[allow(clippy::too_many_arguments)]
pub fn initialize(
    env: Env,
    a: u128,
    token_a: Address,
    token_b: Address,
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<(), Error> {
    require!(!Pool::has(&env), Error::Initialized);

    Pool::from_init_params(&env, a, token_a, token_b, fee_share_bp, admin_fee_share_bp).save(&env);

    Ok(())
}
