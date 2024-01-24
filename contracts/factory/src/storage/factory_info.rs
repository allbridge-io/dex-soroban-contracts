use proc_macros::{
    data_storage_type, extend_ttl_info_instance, SorobanData, SorobanSimpleData, SymbolKey,
};
use shared::{
    utils::{bytes::address_to_bytes, merge_slices_by_half},
    Error,
};
use soroban_sdk::{contracttype, Address, BytesN, Env, Map};

#[contracttype]
#[derive(SorobanData, SorobanSimpleData, SymbolKey)]
#[data_storage_type(Instance)]
#[extend_ttl_info_instance]
pub struct FactoryInfo {
    pub wasm_hash: soroban_sdk::BytesN<32>,
    /// token0 + token1 => pool
    pub pairs: Map<BytesN<64>, Address>,
}

impl FactoryInfo {
    pub fn new(env: &Env, wasm_hash: BytesN<32>) -> Self {
        FactoryInfo {
            wasm_hash,
            pairs: Map::new(env),
        }
    }

    pub fn sort_tokens(token_a: Address, token_b: Address) -> (Address, Address) {
        if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        }
    }

    pub fn merge_addresses(
        env: &Env,
        token_a: &Address,
        token_b: &Address,
    ) -> Result<BytesN<64>, Error> {
        Ok(BytesN::from_array(
            env,
            &merge_slices_by_half::<32, 64>(
                &address_to_bytes(env, &token_a)?.to_array(),
                &address_to_bytes(env, &token_b)?.to_array(),
            ),
        ))
    }

    pub fn add_pair(&mut self, key: BytesN<64>, pool: &Address) {
        self.pairs.set(key.clone(), pool.clone());
    }

    pub fn get_pool(
        &self,
        env: &Env,
        token_a: &Address,
        token_b: &Address,
    ) -> Result<Address, Error> {
        let (token_a, token_b) = FactoryInfo::sort_tokens(token_a.clone(), token_b.clone());
        let bytes = Self::merge_addresses(env, &token_a, &token_b)?;

        self.pairs.get(bytes).ok_or(Error::NotFound)
    }
}
