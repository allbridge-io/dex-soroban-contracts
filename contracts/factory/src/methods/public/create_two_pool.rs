use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};
use storage::Admin;

use crate::storage::factory_info::{FactoryInfo, MAX_PAIRS_NUM};

#[allow(clippy::too_many_arguments)]
pub fn create_two_pool(
    env: Env,
    deployer: Address,
    pool_admin: Address,
    a: u128,
    token_a: Address,
    token_b: Address,
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<Address, Error> {
    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    Admin::require_exist_auth(&env)?;

    let mut factory_info = FactoryInfo::get(&env)?;

    require!(
        factory_info.two_pools.len() < MAX_PAIRS_NUM,
        Error::MaxPoolsNumReached
    );
    require!(token_a != token_b, Error::IdenticalAddresses);
    require!(
        factory_info.get_two_pool(&token_a, &token_b).is_err(),
        Error::PoolExist
    );

    let (token_a, token_b) = FactoryInfo::sort_tokens(token_a, token_b);
    let bytes = FactoryInfo::merge_two_addresses(&token_a, &token_b)?;
    let salt = env.crypto().keccak256(&bytes.into());

    let deployed_pool = env
        .deployer()
        .with_address(deployer, salt)
        .deploy(factory_info.two_pool_wasm_hash.clone());

    factory_info.add_two_pool((token_a.clone(), token_b.clone()), &deployed_pool);

    let args = vec![
        &env,
        *pool_admin.as_val(),
        a.into_val(&env),
        *token_a.as_val(),
        *token_b.as_val(),
        fee_share_bp.into_val(&env),
        admin_fee_share_bp.into_val(&env),
    ];
    env.invoke_contract::<()>(&deployed_pool, &Symbol::new(&env, "initialize"), args);

    factory_info.save(&env);

    Ok(deployed_pool)
}
