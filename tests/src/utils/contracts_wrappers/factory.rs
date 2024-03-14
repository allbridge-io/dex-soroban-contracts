use soroban_sdk::{Address, BytesN, Env};

use crate::{
    contracts::{factory, pool},
    utils::{desoroban_result, unwrap_call_result},
};

pub struct PoolFactory {
    pub id: soroban_sdk::Address,
    pub client: factory::Client<'static>,
    pub env: Env,
}

impl PoolFactory {
    pub fn create(env: &Env, admin: &Address) -> PoolFactory {
        let wasm_hash = env.deployer().upload_contract_wasm(pool::WASM);
        let id = env.register_contract_wasm(None, factory::WASM);
        let client = factory::Client::new(env, &id);

        client.initialize(&wasm_hash, admin);

        PoolFactory {
            id,
            client,
            env: env.clone(),
        }
    }

    pub fn create_pair(
        &self,
        admin: &Address,
        a: u128,
        token_a: &Address,
        token_b: &Address,
        fee_share_bp: u128,
        admin_fee: u128,
    ) -> Address {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_create_pair(
                admin,
                admin,
                &a,
                token_a,
                token_b,
                &fee_share_bp,
                &admin_fee,
            )),
        )
    }

    pub fn update_wasm_hash(&self, new_wasm_hash: &BytesN<32>) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_update_wasm_hash(new_wasm_hash)),
        );
    }

    pub fn set_admin(&self, admin: Address) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_set_admin(&admin)),
        );
    }

    pub fn pool(&self, token_a: &Address, token_b: &Address) -> Address {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_pool(token_a, token_b)),
        )
    }
}
