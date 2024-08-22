use proc_macros::{extend_ttl_info, Persistent, SorobanData};
use shared::consts::DAY_IN_LEDGERS;
use shared::soroban_data::SorobanData;
use soroban_sdk::{contracttype, Address, Env};

use super::sized_array::SizedU128Array;

const BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const LIFETIME_THRESHOLD: u32 = BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone, Debug, SorobanData, Persistent)]
#[extend_ttl_info(BUMP_AMOUNT, LIFETIME_THRESHOLD)]
pub struct UserDeposit {
    pub lp_amount: u128,
    pub reward_debts: SizedU128Array,
}

impl UserDeposit {
    pub fn default_val<const N: usize>(env: &Env) -> Self {
        Self {
            lp_amount: 0,
            reward_debts: SizedU128Array::from_array(env, [0; N]),
        }
    }

    pub fn get<const N: usize>(env: &Env, address: Address) -> UserDeposit {
        UserDeposit::get_by_key(env, &address)
            .unwrap_or_else(|_| UserDeposit::default_val::<N>(env))
    }

    pub fn save(&self, env: &Env, address: Address) {
        self.save_by_key(env, &address);
    }
}
