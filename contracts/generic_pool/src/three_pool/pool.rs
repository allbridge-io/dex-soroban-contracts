use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use soroban_sdk::contracttype;

use crate::{pool::PoolStorage, storage::sized_array::*};

#[contracttype]
#[derive(Debug, Clone, SorobanData, SorobanSimpleData, SymbolKey, Instance)]
#[extend_ttl_info_instance]
pub struct ThreePool {
    pub a: u128,

    pub fee_share_bp: u128,
    pub admin_fee_share_bp: u128,
    pub total_lp_amount: u128,
    pub tokens: SizedAddressArray,
    pub tokens_decimals: SizedDecimalsArray,
    pub token_balances: SizedU128Array,
    pub acc_rewards_per_share_p: SizedU128Array,
    pub admin_fee_amount: SizedU128Array,
}

impl PoolStorage for ThreePool {
    fn a(&self) -> u128 {
        self.a
    }
    fn fee_share_bp(&self) -> u128 {
        self.fee_share_bp
    }
    fn admin_fee_share_bp(&self) -> u128 {
        self.admin_fee_share_bp
    }
    fn total_lp_amount(&self) -> u128 {
        self.total_lp_amount
    }
    fn tokens(&self) -> &SizedAddressArray {
        &self.tokens
    }
    fn tokens_decimals(&self) -> &SizedDecimalsArray {
        &self.tokens_decimals
    }
    fn token_balances(&self) -> &SizedU128Array {
        &self.token_balances
    }
    fn acc_rewards_per_share_p(&self) -> &SizedU128Array {
        &self.acc_rewards_per_share_p
    }
    fn admin_fee_amount(&self) -> &SizedU128Array {
        &self.admin_fee_amount
    }

    fn a_mut(&mut self) -> &mut u128 {
        &mut self.a
    }
    fn fee_share_bp_mut(&mut self) -> &mut u128 {
        &mut self.fee_share_bp
    }
    fn admin_fee_share_bp_mut(&mut self) -> &mut u128 {
        &mut self.admin_fee_share_bp
    }
    fn total_lp_amount_mut(&mut self) -> &mut u128 {
        &mut self.total_lp_amount
    }
    fn tokens_mut(&mut self) -> &mut SizedAddressArray {
        &mut self.tokens
    }
    fn tokens_decimals_mut(&mut self) -> &mut SizedDecimalsArray {
        &mut self.tokens_decimals
    }
    fn token_balances_mut(&mut self) -> &mut SizedU128Array {
        &mut self.token_balances
    }
    fn acc_rewards_per_share_p_mut(&mut self) -> &mut SizedU128Array {
        &mut self.acc_rewards_per_share_p
    }
    fn admin_fee_amount_mut(&mut self) -> &mut SizedU128Array {
        &mut self.admin_fee_amount
    }
}
