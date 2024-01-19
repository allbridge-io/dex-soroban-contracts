use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::Env;

use crate::{pool, storage::factory_info::FactoryInfo};

pub fn initialize(env: Env) -> Result<(), Error> {
    require!(!FactoryInfo::has(&env), Error::Initialized);

    let wasm_hash = env.deployer().upload_contract_wasm(pool::WASM);

    FactoryInfo::new(&env, wasm_hash).save(&env);

    Ok(())
}
