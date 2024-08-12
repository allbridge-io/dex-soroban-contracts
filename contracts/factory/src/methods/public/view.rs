use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, BytesN, Env, Map};
use storage::Admin;

use crate::storage::factory_info::FactoryInfo;

pub fn get_three_pool(env: Env, token_a: &Address, token_b: &Address, token_c: &Address) -> Result<Address, Error> {
    FactoryInfo::get(&env)?.get_three_pool(token_a, token_b, token_c)
}

pub fn get_three_pools(env: Env) -> Result<Map<Address, (Address, Address, Address)>, Error> {
    FactoryInfo::get(&env)?.get_three_pools()
}

pub fn get_two_pool(env: Env, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
    FactoryInfo::get(&env)?.get_two_pool(token_a, token_b)
}

pub fn get_two_pools(env: Env) -> Result<Map<Address, (Address, Address)>, Error> {
    FactoryInfo::get(&env)?.get_two_pools()
}

pub fn get_wasm_hash(env: Env) -> Result<BytesN<32>, Error> {
    Ok(FactoryInfo::get(&env)?.wasm_hash)
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.0)
}
