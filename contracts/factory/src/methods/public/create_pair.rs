use shared::{
    require,
    soroban_data::SimpleSorobanData,
    utils::{bytes::address_to_bytes, merge_slices_by_half},
    Error,
};
use soroban_sdk::{map, Address, Bytes, Env};

use crate::{methods::public::get_pool, pool, storage::factory_info::FactoryInfo};

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
        get_pool(&env, &token_a, &token_b).is_err(),
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
        .deploy(factory_info.wasm_hash);

    factory_info.pairs.set(
        token_a.clone(),
        if let Some(mut map) = factory_info.pairs.get(token_a.clone()) {
            map.set(token_b.clone(), deployed_pool.clone());
            map
        } else {
            map![&env, (token_b.clone(), deployed_pool.clone())]
        },
    );

    pool::Client::new(&env, &deployed_pool).initialize(
        &pool_admin,
        &a,
        &token_a,
        &token_b,
        &fee_share_bp,
        &admin_fee_share_bp,
    );

    Ok(deployed_pool)
}
