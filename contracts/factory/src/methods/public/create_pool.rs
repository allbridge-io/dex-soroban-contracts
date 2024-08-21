use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{vec, Address, Env, IntoVal, Symbol, Vec};
use storage::Admin;

use crate::storage::factory_info::{FactoryInfo, MAX_PAIRS_NUM};

#[allow(clippy::too_many_arguments)]
pub fn create_pool<const N: usize>(
    env: Env,
    deployer: Address,
    pool_admin: Address,
    a: u128,
    tokens: Vec<Address>,
    fee_share_bp: u128,
    admin_fee_share_bp: u128,
) -> Result<Address, Error> {
    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    Admin::require_exist_auth(&env)?;

    let mut factory_info = FactoryInfo::get(&env)?;

    require!(
        factory_info.pools.len() < MAX_PAIRS_NUM,
        Error::MaxPoolsNumReached
    );
    require!(tokens.len() as usize == N, Error::InvalidNumberOfTokens);

    require!(
        factory_info.get_pool(tokens.clone()).is_err(),
        Error::PoolExist
    );

    for (i, token_a) in tokens.iter().enumerate() {
        require!(
            tokens.iter().skip(i + 1).all(|token_b| token_a != token_b),
            Error::IdenticalAddresses
        );
    }

    let sorted_tokens = FactoryInfo::sort_tokens(tokens.clone());
    let mut tokens_with_address = tokens.clone();
    tokens_with_address.push_front(env.current_contract_address());
    let bytes = FactoryInfo::merge_addresses(tokens_with_address)?;
    let salt = env.crypto().keccak256(&bytes);

    let wasm_hash = factory_info.get_pool_wasm_hash::<N>();

    let deployed_pool = env
        .deployer()
        .with_address(deployer, salt)
        .deploy(wasm_hash);

    factory_info.add_pool(sorted_tokens.clone(), &deployed_pool);

    let mut tokens_vals = vec![&env];
    for token in tokens.iter() {
        tokens_vals.push_back(*token.as_val());
    }

    let mut args = vec![&env, *pool_admin.as_val(), a.into_val(&env)];
    args.append(&tokens_vals);
    args.append(&vec![
        &env,
        fee_share_bp.into_val(&env),
        admin_fee_share_bp.into_val(&env),
    ]);

    env.invoke_contract::<()>(&deployed_pool, &Symbol::new(&env, "initialize"), args);

    factory_info.save(&env);

    Ok(deployed_pool)
}
