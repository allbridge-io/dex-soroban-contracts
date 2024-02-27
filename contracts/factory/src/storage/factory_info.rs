use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use shared::{
    utils::{bytes::address_to_bytes, merge_slices_by_half},
    Error,
};
use soroban_sdk::{contracttype, Address, BytesN, Map};

#[contracttype]
#[derive(SorobanData, SorobanSimpleData, SymbolKey, Instance)]
#[extend_ttl_info_instance]
pub struct FactoryInfo {
    pub wasm_hash: soroban_sdk::BytesN<32>,
    /// (token0, token1) => pool
    pub pairs: Map<(Address, Address), Address>,
}

impl FactoryInfo {
    pub fn new(wasm_hash: BytesN<32>) -> Self {
        FactoryInfo {
            wasm_hash: wasm_hash.clone(),
            pairs: Map::new(wasm_hash.env()),
        }
    }

    pub fn sort_tokens(token_a: Address, token_b: Address) -> (Address, Address) {
        if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        }
    }

    pub fn merge_addresses(token_a: &Address, token_b: &Address) -> Result<BytesN<64>, Error> {
        let env = token_a.env();

        Ok(BytesN::from_array(
            env,
            &merge_slices_by_half::<32, 64>(
                &address_to_bytes(env, token_a)?.to_array(),
                &address_to_bytes(env, token_b)?.to_array(),
            ),
        ))
    }

    pub fn add_pair(&mut self, tokens: (Address, Address), pool: &Address) {
        self.pairs.set(tokens, pool.clone());
    }

    pub fn get_pool(&self, token_a: &Address, token_b: &Address) -> Result<Address, Error> {
        let (token_a, token_b) = FactoryInfo::sort_tokens(token_a.clone(), token_b.clone());

        self.pairs.get((token_a, token_b)).ok_or(Error::NotFound)
    }

    pub fn get_pools(&self) -> Result<Map<Address, (Address, Address)>, Error> {
        let mut map = Map::new(self.pairs.env());

        self.pairs.iter().for_each(|(tokens, pool)| {
            map.set(pool, tokens);
        });

        Ok(map)
    }
}
