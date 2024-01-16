use proc_macros::{
    data_storage_type, extend_ttl_info_instance, symbol_key, SorobanData, SorobanSimpleData,
};
use soroban_sdk::{
    contracttype,
    token::{self, TokenClient},
    vec, Address, Env, Vec,
};

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum Token {
    A,
    B,
}

#[contracttype]
#[derive(Debug, Clone, SorobanData, SorobanSimpleData)]
#[symbol_key("Pool")]
#[data_storage_type(Instance)]
#[extend_ttl_info_instance]
pub struct Pool {
    pub a: u128,
    pub fee_share_bp: u128,

    pub tokens: Vec<Address>,
    pub token_balances: Vec<u128>,
    pub acc_rewards_per_share_p: Vec<u128>,

    pub total_lp_amount: u128,
    pub admin_fee_share_bp: u128,
    pub admin_fee_amount: u128,
}

impl Pool {
    pub fn from_init_params(
        env: &Env,
        a: u128,
        token_a: Address,
        token_b: Address,
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self {
        Pool {
            a,
            fee_share_bp,
            admin_fee_share_bp,

            tokens: vec![env, token_a, token_b],
            token_balances: vec![env, 0, 0],
            acc_rewards_per_share_p: vec![env, 0, 0],

            total_lp_amount: 0,
            admin_fee_amount: 0,
        }
    }

    #[inline]
    pub fn get_token_by_index(&self, env: &Env, index: usize) -> TokenClient<'_> {
        token::Client::new(env, &self.tokens.get_unchecked(index as u32))
    }
}
