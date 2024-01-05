use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use crate::{Admin, GasOracleAddress, GasUsage, StopAuthority};

pub fn get_gas_oracle(env: Env) -> Result<Address, Error> {
    Ok(GasOracleAddress::get(&env)?.as_address())
}

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.as_address())
}

pub fn get_stop_authority(env: Env) -> Result<Address, Error> {
    Ok(StopAuthority::get(&env)?.as_address())
}

pub fn get_gas_usage(env: Env, chain_id: u32) -> Result<u128, Error> {
    GasUsage::get_gas_usage_with_default(env, chain_id)
}
