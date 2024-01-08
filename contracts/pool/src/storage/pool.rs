use proc_macros::{
    data_storage_type, extend_ttl_info_instance, symbol_key, SorobanData, SorobanSimpleData,
};
use soroban_sdk::{
    contracttype,
    token::{self, TokenClient},
    Address, Env,
};

#[derive(Debug, Clone, Copy)]
pub enum Tokens {
    TokenA,
    TokenB,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, SorobanData, SorobanSimpleData)]
#[symbol_key("Pool")]
#[data_storage_type(Instance)]
#[extend_ttl_info_instance]
pub struct Pool {
    pub a: u128,
    pub token_a: Address,
    pub token_b: Address,
    pub lp_token: Address,

    pub fee_share_bp: u128,
    pub balance_ratio_min_bp: u128,
    pub d: u128,

    pub token_a_balance: u128,
    pub token_b_balance: u128,

    pub reserves: u128,

    pub decimals_a: u32,
    pub decimals_b: u32,
    pub decimals_lp: u32,

    pub total_lp_amount: u128,
    pub admin_fee_share_bp: u128,
    pub acc_reward_per_share_p: u128,
    pub admin_fee_amount: u128,

    pub can_deposit: bool,
    pub can_withdraw: bool,
}

impl Pool {
    pub fn from_init_params(
        a: u128,
        token_a: Address,
        token_b: Address,
        lp_token: Address,
        fee_share_bp: u128,
        balance_ratio_min_bp: u128,
        admin_fee_share_bp: u128,
        decimals_a: u32,
        decimals_b: u32,
        decimals_lp: u32,
    ) -> Self {
        Pool {
            a,
            token_a,
            token_b,
            lp_token,
            fee_share_bp,
            balance_ratio_min_bp,
            admin_fee_share_bp,
            decimals_a,
            decimals_b,
            decimals_lp,
            can_deposit: true,
            can_withdraw: true,
            d: 0,
            token_a_balance: 0,
            token_b_balance: 0,
            reserves: 0,
            total_lp_amount: 0,
            acc_reward_per_share_p: 0,
            admin_fee_amount: 0,
        }
    }

    #[inline(always)]
    pub fn get_token_a(&self, env: &Env) -> TokenClient<'_> {
        token::Client::new(&env, &self.token_a)
    }

    #[inline(always)]
    pub fn get_token_b(&self, env: &Env) -> TokenClient<'_> {
        token::Client::new(&env, &self.token_b)
    }

    #[inline(always)]
    pub fn get_lp_token(&self, env: &Env) -> TokenClient<'_> {
        token::Client::new(&env, &self.lp_token)
    }

    #[inline(always)]
    pub fn get_lp_native_asset(&self, env: &Env) -> token::StellarAssetClient<'_> {
        token::StellarAssetClient::new(env, &self.lp_token)
    }

    #[inline]
    pub fn get_token_balance(&self, token: Tokens) -> u128 {
        match token {
            Tokens::TokenA => self.token_a_balance,
            Tokens::TokenB => self.token_b_balance,
        }
    }

    #[inline]
    pub fn get_token_client(&self, env: &Env, token: Tokens) -> TokenClient<'_> {
        match token {
            Tokens::TokenA => self.get_token_a(env),
            Tokens::TokenB => self.get_token_b(env),
        }
    }

    #[inline]
    pub fn set_token_balance(&mut self, new_val: u128, token: Tokens) {
        match token {
            Tokens::TokenA => self.token_a_balance = new_val,
            Tokens::TokenB => self.token_b_balance = new_val,
        }
    }
}
