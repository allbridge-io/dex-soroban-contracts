use crate::contracts::three_pool::Token as PoolToken;
use crate::utils::float_to_uint;
use soroban_sdk::{token, Address, Env};

use super::User;

pub struct Token {
    pub id: soroban_sdk::Address,
    pub client: token::Client<'static>,
    pub asset_client: token::StellarAssetClient<'static>,
    pub env: Env,
    pub pool_token: PoolToken,
    pub tag: String,
}

impl Token {
    pub const DEFAULT_AIRDROP: f64 = 100_000_000.0;

    pub fn as_address(&self) -> Address {
        self.id.clone()
    }

    pub fn create(env: &Env, admin: &Address, pool_token: PoolToken, tag: &str) -> Token {
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
