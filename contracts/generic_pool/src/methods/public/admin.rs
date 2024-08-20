use shared::{require, soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{Address, Env};
use storage::Admin;

use crate::common::Pool;

pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;
    Admin(new_admin).save(&env);

    Ok(())
}

pub fn set_fee_share<const N: usize, P: Pool<N>>(
    env: Env,
    fee_share_bp: u128,
) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    require!(fee_share_bp < P::BP, Error::InvalidArg);

    P::update(&env, |pool| {
        *pool.fee_share_bp_mut() = fee_share_bp;
        Ok(())
    })
}

pub fn set_admin_fee_share<const N: usize, P: Pool<N>>(
    env: Env,
    admin_fee_share_bp: u128,
) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    require!(admin_fee_share_bp < P::BP, Error::InvalidArg);

    P::update(&env, |pool| {
        *pool.admin_fee_share_bp_mut() = admin_fee_share_bp;
        Ok(())
    })
}
