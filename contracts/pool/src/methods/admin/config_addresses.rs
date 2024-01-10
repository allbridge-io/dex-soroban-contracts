use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};

use crate::storage::{admin::Admin, stop_authority::StopAuthority};

pub fn set_stop_authority(env: Env, stop_authority: Address) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;
    StopAuthority(stop_authority).save(&env);

    Ok(())
}

pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;
    Admin(new_admin).save(&env);

    Ok(())
}
