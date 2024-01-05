use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{token, Address, Env};

use crate::storage::{admin::Admin, bridge_address::Bridge, pool::Pool};

#[allow(clippy::too_many_arguments)]
pub fn initialize(
    env: Env,
    admin: Address,
    bridge: Address,
    a: u128,
    token_a: Address,
    token_b: Address,
    lp_token: Address,
    fee_share_bp: u128,
    balance_ratio_min_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<(), Error> {
    require!(!Pool::has(&env), Error::Initialized);

    let token_a_client = token::Client::new(&env, &token_a);
    let token_b_client = token::Client::new(&env, &token_b);

    let decimals_a = token_a_client.decimals();
    let decimals_b = token_b_client.decimals();

    Pool::from_init_params(
        a,
        token_a,
        token_b,
        lp_token,
        fee_share_bp,
        balance_ratio_min_bp,
        admin_fee_share_bp,
        decimals_a,
        decimals_b,
    )
    .save(&env);
    Admin(admin).save(&env);
    Bridge(bridge).save(&env);

    Ok(())
}
