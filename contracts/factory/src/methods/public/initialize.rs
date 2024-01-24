use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};
use storage::Admin;

use crate::{
    pool::{self},
    storage::factory_info::FactoryInfo,
};

pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
    require!(!FactoryInfo::has(&env), Error::Initialized);

    let wasm_hash = env.deployer().upload_contract_wasm(pool::WASM);

    FactoryInfo::new(&env, wasm_hash).save(&env);
    Admin(admin).save(&env);

    Ok(())
}
