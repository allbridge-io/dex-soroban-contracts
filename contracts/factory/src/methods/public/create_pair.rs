use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};
use storage::Admin;

use crate::{pool, storage::factory_info::FactoryInfo};

#[allow(clippy::too_many_arguments)]
pub fn create_pair(
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

    require!(token_a != token_b, Error::IdenticalAddresses);
    require!(
        factory_info.get_pool(&token_a, &token_b).is_err(),
        Error::PairExist
    );

    let (token_a, token_b) = FactoryInfo::sort_tokens(token_a, token_b);
    let bytes = FactoryInfo::merge_addresses(&token_a, &token_b)?;
    let salt = env.crypto().keccak256(&bytes.clone().into());

    let deployed_pool = env
        .deployer()
        .with_address(deployer, salt)
        .deploy(factory_info.wasm_hash.clone());

    factory_info.add_pair((token_a.clone(), token_b.clone()), &deployed_pool);

    pool::Client::new(&env, &deployed_pool).initialize(
        &pool_admin,
        &a,
        &token_a,
        &token_b,
        &fee_share_bp,
        &admin_fee_share_bp,
    );

    factory_info.save(&env);

    Ok(deployed_pool)
}
