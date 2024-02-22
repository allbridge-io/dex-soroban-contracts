use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, BytesN, Env};
use storage::Admin;

use crate::storage::factory_info::FactoryInfo;

pub fn initialize(env: Env, wasm_hash: BytesN<32>, admin: Address) -> Result<(), Error> {
    require!(!FactoryInfo::has(&env), Error::Initialized);

    FactoryInfo::new(wasm_hash).save(&env);
    Admin(admin).save(&env);

    Ok(())
}
