use soroban_sdk::{Address, Env};

use crate::{
    contracts::factory,
    utils::{desoroban_result, CallResult},
};

pub struct PoolFactory {
    pub id: soroban_sdk::Address,
    pub client: factory::Client<'static>,
}

impl PoolFactory {
    pub fn create(env: &Env) -> PoolFactory {
        let id = env.register_contract_wasm(None, factory::WASM);
        let client = factory::Client::new(env, &id);

        client.initialize();

        PoolFactory { id, client }
    }

    pub fn create_pair(
        &self,
        admin: &Address,
        a: u128,
        token_a: &Address,
        token_b: &Address,
        fee_share_bp: u128,
        admin_fee: u128,
    ) -> CallResult<Address> {
        desoroban_result(self.client.try_create_pair(
            admin,
            admin,
            &a,
            token_a,
            token_b,
            &fee_share_bp,
            &admin_fee,
        ))
    }

    pub fn get_pool(&self, token_a: &Address, token_b: &Address) -> CallResult<Address> {
        desoroban_result(self.client.try_pool(token_a, token_b))
    }
}
