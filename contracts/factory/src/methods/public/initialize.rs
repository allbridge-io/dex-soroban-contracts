use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, BytesN, Env};
use storage::Admin;

use crate::storage::factory_info::FactoryInfo;

pub fn initialize(env: Env, two_pool_wasm_hash: BytesN<32>, three_pool_wasm_hash: BytesN<32>, admin: Address) -> Result<(), Error> {
    require!(!FactoryInfo::has(&env), Error::Initialized);

    FactoryInfo::new(&env, two_pool_wasm_hash, three_pool_wasm_hash).save(&env);
    Admin(admin).save(&env);

    Ok(())
}
