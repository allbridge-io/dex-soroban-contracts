use soroban_sdk::{vec, Address, BytesN, Env, Vec};

use crate::{
    contracts::{factory, three_pool, two_pool},
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
        let two_pool_wasm_hash = env.deployer().upload_contract_wasm(two_pool::WASM);
        let id = env.register_contract_wasm(None, factory::WASM);
        let client = factory::Client::new(env, &id);

        client.initialize(&two_pool_wasm_hash, &three_pool_wasm_hash, admin);

        PoolFactory {
            id,
            client,
            env: env.clone(),
        }
    }

    pub fn create_pool<const N: usize>(
        &self,
        admin: &Address,
        a: u128,
        tokens: [Address; N],
        fee_share_bp: u128,
        admin_fee: u128,
    ) -> Address {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_create_pool(
                admin,
                admin,
                &a,
                &Vec::from_array(&self.env, tokens),
                &fee_share_bp,
                &admin_fee,
            )),
        )
    }

    pub fn create_three_pool(
        &self,
        admin: &Address,
        a: u128,
        token_a: &Address,
        token_b: &Address,
        token_c: &Address,
        fee_share_bp: u128,
        admin_fee: u128,
    ) -> Address {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_create_pool(
                admin,
                admin,
                &a,
                &vec![&self.env, token_a.clone(), token_b.clone(), token_c.clone()],
                &fee_share_bp,
                &admin_fee,
            )),
        )
    }

    pub fn update_two_wasm_hash(&self, new_wasm_hash: &BytesN<32>) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_update_two_pool_wasm_hash(new_wasm_hash)),
        );
    }

    pub fn update_three_wasm_hash(&self, new_wasm_hash: &BytesN<32>) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_update_three_pool_wasm_hash(new_wasm_hash)),
        );
    }

    pub fn set_admin(&self, admin: Address) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_set_admin(&admin)),
        );
    }

    pub fn pool<const N: usize>(&self, tokens: [Address; N]) -> Address {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_pool(&Vec::from_array(&self.env, tokens))),
        )
    }
}
