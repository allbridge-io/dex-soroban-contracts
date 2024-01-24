use shared::{soroban_data::SimpleSorobanData, utils::extend_ttl_instance, Error};
use soroban_sdk::{Address, Env};

use crate::storage::factory_info::FactoryInfo;

pub fn get_pool(env: Env, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
    extend_ttl_instance(&env);

    FactoryInfo::get(&env)?.get_pool(&env, token_a, token_b)
}
