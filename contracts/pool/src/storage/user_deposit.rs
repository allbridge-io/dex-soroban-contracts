use proc_macros::{extend_ttl_info, Persistent, SorobanData};
use shared::consts::DAY_IN_LEDGERS;
use shared::soroban_data::SorobanData;
use soroban_sdk::{contracttype, Address, Env};

use super::double_values::DoubleU128;

const BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone, Debug, SorobanData, Default, Persistent)]
#[extend_ttl_info(BUMP_AMOUNT, LIFETIME_THRESHOLD)]
pub struct UserDeposit {
    pub lp_amount: u128,
    pub reward_debts: DoubleU128,
}

impl UserDeposit {
    pub fn get(env: &Env, address: Address) -> UserDeposit {
        UserDeposit::get_by_key(env, &address).unwrap_or_default()
    }

    pub fn save(&self, env: &Env, address: Address) {
        self.save_by_key(env, &address);
    }
}
