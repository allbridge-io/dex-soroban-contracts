use proc_macros::{
    data_storage_type, extend_ttl_info_instance, symbol_key, SorobanData, SorobanSimpleData,
};
use soroban_sdk::{
    contracttype,
    token::{self, TokenClient},
    Address, Env,
};

#[derive(Debug, Clone, Copy)]
pub enum Token {
    A,
    B,
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

    pub fee_share_bp: u128,
    pub d: u128,

    pub token_a_balance: u128,
    pub token_b_balance: u128,

    pub total_lp_amount: u128,
    pub admin_fee_share_bp: u128,
    pub acc_reward_a_per_share_p: u128,
    pub acc_reward_b_per_share_p: u128,
    pub admin_fee_amount: u128,
}

impl Pool {
    pub fn from_init_params(
        a: u128,
        token_a: Address,
        token_b: Address,
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self {
        Pool {
            a,
            token_a,
            token_b,
            fee_share_bp,
            admin_fee_share_bp,
            d: 0,
            token_a_balance: 0,
            token_b_balance: 0,
            total_lp_amount: 0,
            acc_reward_a_per_share_p: 0,
            acc_reward_b_per_share_p: 0,
            admin_fee_amount: 0,
        }
    }

    #[inline]
    pub fn get_token_balance(&self, token: Token) -> u128 {
        match token {
            Token::A => self.token_a_balance,
            Token::B => self.token_b_balance,
        }
    }

    #[inline]
    pub fn get_token(&self, env: &Env, token: Token) -> TokenClient<'_> {
        token::Client::new(
            env,
            match token {
                Token::A => &self.token_a,
                Token::B => &self.token_b,
            },
        )
    }

    #[inline]
    pub fn set_token_balance(&mut self, new_val: u128, token: Token) {
        match token {
            Token::A => self.token_a_balance = new_val,
            Token::B => self.token_b_balance = new_val,
        }
    }
}
