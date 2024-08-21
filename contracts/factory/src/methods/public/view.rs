use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, BytesN, Env, Map, Vec};
use storage::Admin;

use crate::storage::factory_info::FactoryInfo;

pub fn get_pool(env: Env, tokens: Vec<Address>) -> Result<Address, Error> {
    FactoryInfo::get(&env)?.get_pool(tokens)
}

pub fn get_pools(env: &Env) -> Result<Map<Address, Vec<Address>>, Error> {
    FactoryInfo::get(env)?.get_pools()
}

pub fn get_pool_wasm_hash<const N: usize>(env: Env) -> Result<BytesN<32>, Error> {
    Ok(FactoryInfo::get(&env)?.get_pool_wasm_hash::<N>())
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.0)
}
