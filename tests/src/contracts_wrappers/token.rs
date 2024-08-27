use soroban_sdk::{token, Address, Env};

use crate::{
    contracts::{three_pool::ThreeToken, two_pool::TwoToken},
    utils::float_to_uint,
};

use super::User;

impl From<TwoToken> for usize {
    fn from(value: TwoToken) -> Self {
        value as usize
    }
}

impl From<ThreeToken> for usize {
    fn from(value: ThreeToken) -> Self {
        value as usize
    }
}

impl From<usize> for TwoToken {
    fn from(value: usize) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
}

impl From<usize> for ThreeToken {
    fn from(value: usize) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
}

pub struct Token<T: Into<usize>> {
    pub id: soroban_sdk::Address,
    pub tag: String,
    pub client: token::Client<'static>,
    pub pool_token: T,
    pub asset_client: token::StellarAssetClient<'static>,
    pub env: Env,
}

impl<T: Into<usize>> Token<T> {
    pub const DEFAULT_AIRDROP: f64 = 100_000_000.0;

    pub fn as_address(&self) -> Address {
        self.id.clone()
    }

    pub fn create(env: &Env, admin: &Address, pool_token: T, tag: &str) -> Token<T> {
        #[allow(deprecated)]
        let id = env.register_stellar_asset_contract(admin.clone());
        let client = token::Client::new(env, &id);
        let asset_client = token::StellarAssetClient::new(env, &id);

        Token {
            id,
            client,
            asset_client,
            env: env.clone(),
            pool_token,
            tag: tag.into(),
        }
    }

    pub fn airdrop(&self, user: &User, amount: f64) {
        self.asset_client.mint(
            user.as_ref(),
            &(float_to_uint(amount, self.client.decimals()) as i128),
        );
    }

    pub fn default_airdrop(&self, user: &User) {
        self.asset_client.mint(
            user.as_ref(),
            &(float_to_uint(Self::DEFAULT_AIRDROP, self.client.decimals()) as i128),
        );
    }

    pub fn balance_of(&self, id: &Address) -> u128 {
        self.client.balance(id) as u128
    }
}
