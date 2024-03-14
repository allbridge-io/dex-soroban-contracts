use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, BytesN, Env, Map};
use storage::Admin;

use crate::storage::factory_info::FactoryInfo;

pub fn get_pool(env: Env, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
    FactoryInfo::get(&env)?.get_pool(token_a, token_b)
}

pub fn get_pools(env: Env) -> Result<Map<Address, (Address, Address)>, Error> {
    FactoryInfo::get(&env)?.get_pools()
}

pub fn get_wasm_hash(env: Env) -> Result<BytesN<32>, Error> {
    Ok(FactoryInfo::get(&env)?.wasm_hash)
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.0)
}
