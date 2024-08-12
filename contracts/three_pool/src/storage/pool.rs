use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use soroban_sdk::{
    contracttype,
    token::{self, TokenClient},
    Address, Env,
};

use super::{
    common::Token,
    triple_values::{TripleAddress, TripleU128, TripleU32},
};

#[contracttype]
#[derive(Debug, Clone, SorobanData, SorobanSimpleData, SymbolKey, Instance)]
#[extend_ttl_info_instance]
pub struct Pool {
    pub a: u128,

    pub fee_share_bp: u128,
    pub admin_fee_share_bp: u128,
    pub total_lp_amount: u128,

    pub tokens: TripleAddress,
    pub tokens_decimals: TripleU32,
    pub token_balances: TripleU128,
    pub acc_rewards_per_share_p: TripleU128,
    pub admin_fee_amount: TripleU128,
}

impl Pool {
    pub fn from_init_params(
        a: u128,
        token_a: Address,
        token_b: Address,
        token_c: Address,
        decimals: (u32, u32, u32),
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self {
        Pool {
            a,

            fee_share_bp,
            admin_fee_share_bp,
            total_lp_amount: 0,

            tokens: TripleAddress::from((token_a, token_b, token_c)),
            tokens_decimals: TripleU32::from(decimals),
            token_balances: TripleU128::default(),
            acc_rewards_per_share_p: TripleU128::default(),
            admin_fee_amount: TripleU128::default(),
        }
    }

    #[inline]
    pub fn get_token_by_index(&self, env: &Env, index: usize) -> TokenClient<'_> {
        token::Client::new(env, &self.tokens[index])
    }

    #[inline]
    pub fn get_token(&self, env: &Env, token: Token) -> TokenClient<'_> {
        self.get_token_by_index(env, token as usize)
    }
}
