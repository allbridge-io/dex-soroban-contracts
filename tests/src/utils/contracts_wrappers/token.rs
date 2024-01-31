use soroban_sdk::{token, Address, Env};

use crate::utils::{float_to_int, int_to_float};

use super::User;

pub struct Token {
    pub id: soroban_sdk::Address,
    pub client: token::Client<'static>,
    pub asset_client: token::StellarAssetClient<'static>,
}

impl Token {
    pub fn as_address(&self) -> Address {
        self.id.clone()
    }

    pub fn create(env: &Env, admin: &Address) -> Token {
        let id = env.register_stellar_asset_contract(admin.clone());
        let client = token::Client::new(env, &id);
        let asset_client = token::StellarAssetClient::new(env, &id);

        Token {
            id,
            client,
            asset_client,
        }
    }

    pub fn clone_token(&self, env: &Env) -> Token {
        let client = token::Client::new(env, &self.id);
        let asset_client = token::StellarAssetClient::new(env, &self.id);

        Token {
            id: self.id.clone(),
            client,
            asset_client,
        }
    }

    pub fn decimals(&self) -> u32 {
        self.client.decimals()
    }

    pub fn airdrop_amount(&self, id: &Address, amount: f64) {
        self.asset_client
            .mint(id, &(float_to_int(amount, self.client.decimals()) as i128));
    }

    pub fn airdrop(&self, id: &Address) {
        self.asset_client.mint(
            id,
            &(float_to_int(75_000.0, self.client.decimals()) as i128),
        );
    }

    pub fn airdrop_user(&self, user: &User) {
        self.airdrop(&user.as_address())
    }

    pub fn balance_of(&self, id: &Address) -> u128 {
        self.client.balance(id) as u128
    }

    pub fn balance_of_f64(&self, id: &Address) -> f64 {
        int_to_float(self.client.balance(id) as u128, 7)
    }
}
