use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use shared::{
    utils::{bytes::address_to_bytes, merge_slices_by_third},
    Error,
};
use soroban_sdk::{contracttype, Address, BytesN, Map};
use shared::utils::merge_slices_by_half;

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

    pub fn sort_tokens(token_a: Address, token_b: Address) -> (Address, Address) {
        if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        }
    }

    pub fn sort_three_tokens(token_a: Address, token_b: Address, token_c: Address) -> (Address, Address, Address) {
        let mut arr = [token_a, token_b, token_c];
        for i in 0..arr.len() {
            for j in 0..arr.len() - 1 - i {
                if arr[j] > arr[j + 1] {
                    arr.swap(j, j + 1);
                }
            }
        }

        (arr[0].clone(), arr[1].clone(), arr[2].clone())
    }

    pub fn merge_two_addresses(token_a: &Address, token_b: &Address) -> Result<BytesN<64>, Error> {
        let env = token_a.env();

        Ok(BytesN::from_array(
            env,
            &merge_slices_by_half::<32, 64>(
                &address_to_bytes(env, token_a)?.to_array(),
                &address_to_bytes(env, token_b)?.to_array(),
            ),
        ))
    }

    pub fn merge_three_addresses(token_a: &Address, token_b: &Address, token_c: &Address) -> Result<BytesN<96>, Error> {
        let env = token_a.env();

        Ok(BytesN::from_array(
            env,
            &merge_slices_by_third::<32, 96>(
                &address_to_bytes(env, token_a)?.to_array(),
                &address_to_bytes(env, token_b)?.to_array(),
                &address_to_bytes(env, token_c)?.to_array(),
            ),
        ))
    }

    pub fn add_three_pool(&mut self, tokens: (Address, Address, Address), pool: &Address) {
        self.three_pools.set(tokens, pool.clone());
    }

    pub fn get_three_pool(&self, token_a: &Address, token_b: &Address, token_c: &Address) -> Result<Address, Error> {
        let (token_a, token_b, token_c) = FactoryInfo::sort_three_tokens(token_a.clone(), token_b.clone(), token_c.clone());

        self.three_pools.get((token_a, token_b, token_c)).ok_or(Error::NotFound)
    }

    pub fn get_three_pools(&self) -> Result<Map<Address, (Address, Address, Address)>, Error> {
        let mut map = Map::new(self.two_pools.env());

        self.three_pools.iter().for_each(|(tokens, pool)| {
            map.set(pool, tokens);
        });

        Ok(map)
    }

    pub fn add_two_pool(&mut self, tokens: (Address, Address), pool: &Address) {
        self.two_pools.set(tokens, pool.clone());
    }

    pub fn get_two_pool(&self, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
        let (token_a, token_b) = FactoryInfo::sort_tokens(token_a.clone(), token_b.clone());

        self.two_pools.get((token_a, token_b)).ok_or(Error::NotFound)
    }

    pub fn get_two_pools(&self) -> Result<Map<Address, (Address, Address)>, Error> {
        let mut map = Map::new(self.two_pools.env());

        self.two_pools.iter().for_each(|(tokens, pool)| {
            map.set(pool, tokens);
        });

        Ok(map)
    }
}
