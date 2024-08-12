use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use shared::{
    utils::{bytes::address_to_bytes},
    Error,
};
use soroban_sdk::{contracttype, vec, Address, Bytes, BytesN, Map, Vec};

pub const MAX_PAIRS_NUM: u32 = 21;

#[contracttype]
#[derive(SorobanData, SorobanSimpleData, SymbolKey, Instance)]
#[extend_ttl_info_instance]
pub struct FactoryInfo {
    pub two_pool_wasm_hash: soroban_sdk::BytesN<32>,
    pub three_pool_wasm_hash: soroban_sdk::BytesN<32>,
    /// (token0, token1) => pool
    pub two_pools: Map<(Address, Address), Address>,
    /// (token0, token1, token2) => pool
    pub three_pools: Map<(Address, Address, Address), Address>,
}

impl FactoryInfo {
    pub fn new(two_pool_wasm_hash: BytesN<32>, three_pool_wasm_hash: BytesN<32>) -> Self {
        FactoryInfo {
            two_pool_wasm_hash: two_pool_wasm_hash.clone(),
            three_pool_wasm_hash: three_pool_wasm_hash.clone(),
            two_pools: Map::new(two_pool_wasm_hash.env()),
            three_pools: Map::new(three_pool_wasm_hash.env()),
        }
    }

    pub fn sort_tokens<const N: usize>(mut arr: [Address; N]) -> [Address; N] {
        for i in 0..arr.len() {
            for j in 0..arr.len() - 1 - i {
                if arr[j] > arr[j + 1] {
                    arr.swap(j, j + 1);
                }
            }
        }

        arr
    }

    pub fn merge_addresses(tokens: Vec<Address>) -> Result<Bytes, Error> {
        let env = tokens.env();
        let mut result = Bytes::new(env);

        for token in tokens.iter() {
            let address_bytes = address_to_bytes(env, &token)?;
            result.extend_from_array(&address_bytes.to_array());
        }

        Ok(result)
    }

    pub fn add_three_pool(&mut self, tokens: (Address, Address, Address), pool: &Address) {
        self.three_pools.set(tokens, pool.clone());
    }

    pub fn get_three_pool(&self, token_a: &Address, token_b: &Address, token_c: &Address) -> Result<Address, Error> {
        let [token_a, token_b, token_c] = FactoryInfo::sort_tokens([token_a.clone(), token_b.clone(), token_c.clone()]);

        self.three_pools.get((token_a, token_b, token_c)).ok_or(Error::NotFound)
    }

    pub fn get_three_pools(&self) -> Result<Map<Address, Vec<Address>>, Error> {
        let mut map = Map::new(self.two_pools.env());

        self.three_pools.iter().for_each(|(tokens, pool)| {
            map.set(pool, vec![&self.two_pools.env(), tokens.0, tokens.1, tokens.2]);
        });

        Ok(map)
    }

    pub fn add_two_pool(&mut self, tokens: (Address, Address), pool: &Address) {
        self.two_pools.set(tokens, pool.clone());
    }

    pub fn get_two_pool(&self, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
        let [token_a, token_b] = FactoryInfo::sort_tokens([token_a.clone(), token_b.clone()]);

        self.two_pools.get((token_a, token_b)).ok_or(Error::NotFound)
    }

    pub fn get_two_pools(&self) -> Result<Map<Address, Vec<Address>>, Error> {
        let mut map = Map::new(self.two_pools.env());

        self.two_pools.iter().for_each(|(tokens, pool)| {
            map.set(pool, vec![&self.two_pools.env(), tokens.0, tokens.1]);
        });

        Ok(map)
    }
}
