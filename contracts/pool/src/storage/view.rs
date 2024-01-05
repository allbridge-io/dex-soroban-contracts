use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use super::{admin::Admin, stop_authority::StopAuthority};

pub fn get_admin(env: Env) -> Result<Address, Error> {
    Ok(Admin::get(&env)?.as_address())
}

pub fn get_stop_authority(env: Env) -> Result<Address, Error> {
    Ok(StopAuthority::get(&env)?.as_address())
}
