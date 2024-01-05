use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::Env;

use crate::storage::{admin::Admin, pool::Pool, stop_authority::StopAuthority};

pub fn stop_deposit(env: Env) -> Result<(), Error> {
    StopAuthority::get(&env)?.require_stop_authority_auth();

    Pool::update(&env, |pool| {
        pool.can_deposit = false;
        Ok(())
    })
}

pub fn start_deposit(env: Env) -> Result<(), Error> {
    // only admin can start deposit, not stop_authority
    Admin::require_exist_auth(&env)?;

    Pool::update(&env, |pool| {
        pool.can_deposit = true;
        Ok(())
    })
}

pub fn stop_withdraw(env: Env) -> Result<(), Error> {
    StopAuthority::get(&env)?.require_stop_authority_auth();

    Pool::update(&env, |pool| {
        pool.can_withdraw = false;
        Ok(())
    })
}

pub fn start_withdraw(env: Env) -> Result<(), Error> {
    // only admin can start withdraw, not stop_authority
    Admin::require_exist_auth(&env)?;

    Pool::update(&env, |pool| {
        pool.can_withdraw = true;
        Ok(())
    })
}
