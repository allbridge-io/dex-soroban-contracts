use soroban_sdk::{Address, Env};

use shared::soroban_data::SimpleSorobanData;
use shared::Error;
use storage::Admin;

pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;
    Admin(new_admin).save(&env);

    Ok(())
}
