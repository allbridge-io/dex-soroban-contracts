use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol};
use storage::Admin;

use crate::storage::factory_info::{FactoryInfo, MAX_PAIRS_NUM};

#[allow(clippy::too_many_arguments)]
pub fn create_three_pool(
    env: Env,
    deployer: Address,
    pool_admin: Address,
    a: u128,
    token_a: Address,
    token_b: Address,
    token_c: Address,
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<Address, Error> {
    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    Admin::require_exist_auth(&env)?;

    let mut factory_info = FactoryInfo::get(&env)?;

    require!(
        factory_info.three_pools.len() < MAX_PAIRS_NUM,
        Error::MaxPoolsNumReached
    );
    require!(token_a != token_b && token_a != token_c && token_b != token_c, Error::IdenticalAddresses);
    require!(
        factory_info.get_three_pool(&token_a, &token_b, &token_c).is_err(),
        Error::PoolExist
    );

    let [token_a, token_b, token_c] = FactoryInfo::sort_tokens([token_a, token_b, token_c]);
    let bytes = FactoryInfo::merge_addresses(vec![&env, env.current_contract_address(), token_a.clone(), token_b.clone(), token_c.clone()])?;
    let salt = env.crypto().keccak256(&bytes);

    let deployed_pool = env
        .deployer()
        .with_address(deployer, salt)
        .deploy(factory_info.three_pool_wasm_hash.clone());

    factory_info.add_three_pool((token_a.clone(), token_b.clone(), token_c.clone()), &deployed_pool);

    let args = vec![
        &env,
        *pool_admin.as_val(),
        a.into_val(&env),
        *token_a.as_val(),
        *token_b.as_val(),
        *token_c.as_val(),
        fee_share_bp.into_val(&env),
        admin_fee_share_bp.into_val(&env),
    ];
    env.invoke_contract::<()>(&deployed_pool, &Symbol::new(&env, "initialize"), args);

    factory_info.save(&env);

    Ok(deployed_pool)
}
