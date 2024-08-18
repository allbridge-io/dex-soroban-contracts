use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use soroban_sdk::{
    contracttype,
    token::{self, TokenClient},
    Address, Env,
};

use crate::storage::sized_array::*;

use super::common::Token;

#[contracttype]
#[derive(Debug, Clone, SorobanData, SorobanSimpleData, SymbolKey, Instance)]
#[extend_ttl_info_instance]
pub struct Pool {
    pub a: u128,

    pub fee_share_bp: u128,
    pub admin_fee_share_bp: u128,
    pub total_lp_amount: u128,
    pub tokens: SizedAddressArray,
    pub tokens_decimals: SizedDecimalsArray,
    pub token_balances: SizedU128Array,
    pub acc_rewards_per_share_p: SizedU128Array,
    pub admin_fee_amount: SizedU128Array,
    // pub tokens: Vec<Address>,
    // pub tokens_decimals: Vec<u32>,
    // pub token_balances: Vec<u128>,
    // pub acc_rewards_per_share_p: Vec<u128>,
    // pub admin_fee_amount: Vec<u128>,
}

impl Pool {
    pub fn from_init_params(
        env: &Env,
        a: u128,
        token_a: Address,
        token_b: Address,
        token_c: Address,
        decimals: [u32; 3],
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self {
        Pool {
            a,

            fee_share_bp,
            admin_fee_share_bp,
            total_lp_amount: 0,

            tokens: SizedAddressArray::from_array(env, [token_a, token_b, token_c]),
            tokens_decimals: SizedDecimalsArray::from_array(env, decimals),
            token_balances: SizedU128Array::from_array(env, [0, 0, 0]),
            acc_rewards_per_share_p: SizedU128Array::from_array(env, [0, 0, 0]),
            admin_fee_amount: SizedU128Array::from_array(env, [0, 0, 0]),
        }
    }

    #[inline]
    pub fn get_token_by_index(&self, env: &Env, index: usize) -> TokenClient<'_> {
        token::Client::new(env, &self.tokens.get(index))
    }

    #[inline]
    pub fn get_token(&self, env: &Env, token: Token) -> TokenClient<'_> {
        self.get_token_by_index(env, token as usize)
    }
}
