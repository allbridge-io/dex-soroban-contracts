use proc_macros::{
    data_storage_type, extend_ttl_info_instance, symbol_key, SorobanData, SorobanSimpleData,
};
use soroban_sdk::{contracttype, Address, Map};

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
