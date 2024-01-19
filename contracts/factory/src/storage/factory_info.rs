use proc_macros::{
    data_storage_type, extend_ttl_info_instance, symbol_key, SorobanData, SorobanSimpleData,
};
use shared::Error;
use soroban_sdk::{contracttype, map, Address, BytesN, Env, Map};

#[contracttype]
#[derive(SorobanData, SorobanSimpleData)]
#[symbol_key("Factory")]
#[data_storage_type(Instance)]
#[extend_ttl_info_instance]
pub struct FactoryInfo {
    pub wasm_hash: soroban_sdk::BytesN<32>,
    /// token0 => token1 => pool
    pub pairs: Map<Address, Map<Address, Address>>,
}

impl FactoryInfo {
    pub fn new(env: &Env, wasm_hash: BytesN<32>) -> Self {
        FactoryInfo {
            wasm_hash,
            pairs: Map::new(&env),
        }
    }

    pub fn add_pair(&mut self, env: &Env, token_a: &Address, token_b: &Address, pool: &Address) {
        self.pairs.set(
            token_a.clone(),
            if let Some(mut map) = self.pairs.get(token_a.clone()) {
                map.set(token_b.clone(), pool.clone());
                map
            } else {
                map![&env, (token_b.clone(), pool.clone())]
            },
        );
    }

    pub fn get_pool(&self, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
        self.pairs
            .get(token_a.clone())
            .and_then(|inner_map| inner_map.get(token_b.clone()))
            .ok_or(Error::NotFound)
    }
}
