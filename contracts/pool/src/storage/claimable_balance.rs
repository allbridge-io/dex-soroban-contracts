use proc_macros::{data_storage_type, SorobanData, extend_ttl_info};
use shared::soroban_data::SorobanData;
use soroban_sdk::{contracttype, Address, Env};
use shared::consts::DAY_IN_LEDGERS;
use shared::Error;

use crate::storage::data_key::DataKey;

const BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Default, SorobanData)]
#[data_storage_type(Persistent)]
#[extend_ttl_info(BUMP_AMOUNT, LIFETIME_THRESHOLD)]
pub struct ClaimableBalance {
    pub amount: u128,
}

impl ClaimableBalance {
    pub fn get(env: &Env, address: Address) -> ClaimableBalance {
        ClaimableBalance::get_by_key(env, &DataKey::ClaimableBalance(address)).unwrap_or_default()
    }

    pub fn save(&self, env: &Env, address: Address) {
        self.save_by_key(env, &DataKey::ClaimableBalance(address));
    }

    pub fn update<F>(env: &Env, address: Address, handler: F) -> Result<(), Error>
        where
            F: FnOnce(&mut Self) -> Result<(), Error>,
    {
        let mut object = Self::get(env, address.clone());

        handler(&mut object)?;
        object.save(env, address);

        Ok(())
    }
}
