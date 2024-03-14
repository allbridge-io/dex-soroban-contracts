use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{BytesN, Env};
use storage::Admin;

use crate::storage::factory_info::FactoryInfo;

pub fn update_wasm_hash(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
    Admin::require_exist_auth(&env)?;

    FactoryInfo::update(&env, |info| {
        info.wasm_hash = new_wasm_hash;

        Ok(())
    })
}
