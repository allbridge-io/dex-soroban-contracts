use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{BytesN, Env};
use storage::Admin;

use crate::storage::factory_info::FactoryInfo;

pub fn update_pool_wasm_hash<const N: usize>(
    env: Env,
    new_wasm_hash: BytesN<32>,
) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    FactoryInfo::update(&env, |info| {
        if N == 2 {
            info.two_pool_wasm_hash = new_wasm_hash;
        } else {
            info.three_pool_wasm_hash = new_wasm_hash;
        };

        Ok(())
    })
}
