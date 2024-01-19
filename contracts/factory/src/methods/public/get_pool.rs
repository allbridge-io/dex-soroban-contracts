use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use crate::storage::factory_info::FactoryInfo;

pub fn get_pool(env: &Env, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
    FactoryInfo::get(env)?
        .pairs
        .get(token_a.clone())
        .and_then(|inner_map| inner_map.get(token_b.clone()))
        .ok_or(Error::NotFound)
}
