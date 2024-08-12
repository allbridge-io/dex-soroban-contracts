use soroban_sdk::{Address, BytesN, Env, vec};

use crate::{
    contracts::{factory, pool, three_pool},
    utils::{desoroban_result, unwrap_call_result},
};

pub struct PoolFactory {
    pub id: soroban_sdk::Address,
    pub client: factory::Client<'static>,
    pub env: Env,
}

impl PoolFactory {
    pub fn create(env: &Env, admin: &Address) -> PoolFactory {
        let three_pool_wasm_hash = env.deployer().upload_contract_wasm(three_pool::WASM);
        let two_pool_wasm_hash = env.deployer().upload_contract_wasm(pool::WASM);
        let id = env.register_contract_wasm(None, factory::WASM);
        let client = factory::Client::new(env, &id);

        client.initialize(&two_pool_wasm_hash, &three_pool_wasm_hash, admin);

        PoolFactory {
            id,
            client,
            env: env.clone(),
        }
    }

    pub fn create_pool(
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
            desoroban_result(self.client.try_create_pool(
                admin,
                admin,
                &a,
                &soroban_sdk::vec![&self.env, token_a.clone(), token_b.clone()],
                &fee_share_bp,
                &admin_fee,
            )),
        )
    }

    pub fn update_wasm_hash(&self, new_wasm_hash: &BytesN<32>) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_update_two_pool_wasm_hash(new_wasm_hash)),
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
            desoroban_result(self.client.try_pool(&vec![&self.env, token_a.clone(), token_b.clone()])),

        )
    }
}
