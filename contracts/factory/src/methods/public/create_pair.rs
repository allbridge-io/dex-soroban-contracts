use shared::{
    require,
    soroban_data::SimpleSorobanData,
    utils::{bytes::address_to_bytes, merge_slices_by_half},
    Error,
};
use soroban_sdk::{Address, Bytes, Env};

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

    let mut factory_info = FactoryInfo::get(&env)?;

    require!(token_a != token_b, Error::IdenticalAddresses);

    let (token_a, token_b) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    require!(
        factory_info.get_pool(&token_a, &token_b).is_err(),
        Error::PairExist
    );

    let bytes = merge_slices_by_half::<32, 64>(
        &address_to_bytes(&env, &token_a)?.to_array(),
        &address_to_bytes(&env, &token_b)?.to_array(),
    );
    let salt = env.crypto().keccak256(&Bytes::from_array(&env, &bytes));

    let deployed_pool = env
        .deployer()
        .with_address(deployer, salt)
        .deploy(factory_info.wasm_hash.clone());

    factory_info.add_pair(&env, &token_a, &token_b, &deployed_pool);
    factory_info.add_pair(&env, &token_b, &token_a, &deployed_pool);

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
