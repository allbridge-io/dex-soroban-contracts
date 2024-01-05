use proc_macros::{
    extend_ttl_info_instance, data_storage_type, symbol_key, SorobanData, SorobanSimpleData,
};
use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{contracttype, symbol_short, token, Address, Env, Symbol};

pub const NATIVE_TOKEN_SYMBOL: Symbol = symbol_short!("NatvTknAd");

#[contracttype]
#[derive(SorobanData, SorobanSimpleData)]
#[symbol_key("NatvTknAd")]
#[data_storage_type(Instance)]
#[extend_ttl_info_instance]
pub struct NativeToken(pub Address);

impl NativeToken {
    #[inline]
    pub fn as_address(&self) -> Address {
        self.0.clone()
    }

    #[inline]
    pub fn get_client(env: &Env) -> Result<token::Client, Error> {
        let address = Self::get(env)?.as_address();
        Ok(token::Client::new(env, &address))
    }
}
