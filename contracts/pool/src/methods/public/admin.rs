use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};
use storage::Admin;

use crate::storage::pool::Pool;

pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;
    Admin(new_admin).save(&env);

    Ok(())
}

pub fn set_fee_share(env: Env, fee_share_bp: u128) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    require!(fee_share_bp < Pool::BP, Error::InvalidArg);

    Pool::update(&env, |pool| {
        pool.fee_share_bp = fee_share_bp;
        Ok(())
    })
}

pub fn set_admin_fee_share(env: Env, admin_fee_share_bp: u128) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    require!(admin_fee_share_bp < Pool::BP, Error::InvalidArg);

    Pool::update(&env, |pool| {
        pool.admin_fee_share_bp = admin_fee_share_bp;
        Ok(())
    })
}
