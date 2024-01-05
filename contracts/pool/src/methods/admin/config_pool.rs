use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::Env;

use crate::storage::{admin::Admin, pool::Pool};

pub fn set_fee_share(env: Env, fee_share_bp: u128) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    Pool::update(&env, |pool| {
        pool.fee_share_bp = fee_share_bp;
        Ok(())
    })
}

pub fn set_balance_ratio_min_bp(env: Env, balance_ratio_min_bp: u128) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    Pool::update(&env, |pool| {
        pool.balance_ratio_min_bp = balance_ratio_min_bp;
        Ok(())
    })
}

pub fn set_admin_fee_share(env: Env, admin_fee_share_bp: u128) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    Pool::update(&env, |pool| {
        pool.admin_fee_share_bp = admin_fee_share_bp;
        Ok(())
    })
}
