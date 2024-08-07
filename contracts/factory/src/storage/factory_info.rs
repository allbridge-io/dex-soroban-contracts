use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use shared::{
    utils::{bytes::address_to_bytes, merge_slices_by_third},
    Error,
};
use soroban_sdk::{contracttype, Address, BytesN, Map};

pub const MAX_PAIRS_NUM: u32 = 21;

#[contracttype]
#[derive(SorobanData, SorobanSimpleData, SymbolKey, Instance)]
#[extend_ttl_info_instance]
pub struct FactoryInfo {
    pub wasm_hash: soroban_sdk::BytesN<32>,
    /// (token0, token1) => pool
    pub pools: Map<(Address, Address, Address), Address>,
}

impl FactoryInfo {
    pub fn new(wasm_hash: BytesN<32>) -> Self {
        FactoryInfo {
            wasm_hash: wasm_hash.clone(),
            pools: Map::new(wasm_hash.env()),
        }
    }

    pub fn sort_tokens(token_a: Address, token_b: Address, token_c: Address) -> (Address, Address, Address) {
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

    pub fn merge_addresses(token_a: &Address, token_b: &Address, token_c: &Address) -> Result<BytesN<96>, Error> {
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

    pub fn add_pool(&mut self, tokens: (Address, Address, Address), pool: &Address) {
        self.pools.set(tokens, pool.clone());
    }

    pub fn get_pool(&self, token_a: &Address, token_b: &Address, token_c: &Address) -> Result<Address, Error> {
        let (token_a, token_b, token_c) = FactoryInfo::sort_tokens(token_a.clone(), token_b.clone(), token_c.clone());

        self.pools.get((token_a, token_b, token_c)).ok_or(Error::NotFound)
    }

    pub fn get_pools(&self) -> Result<Map<Address, (Address, Address, Address)>, Error> {
        let mut map = Map::new(self.pools.env());

        self.pools.iter().for_each(|(tokens, pool)| {
            map.set(pool, tokens);
        });

        Ok(map)
    }
}
