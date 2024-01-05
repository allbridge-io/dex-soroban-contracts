use proc_macros::{
    extend_ttl_info_instance, data_storage_type, symbol_key, SorobanData, SorobanSimpleData,
};
use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
#[derive(SorobanData, SorobanSimpleData)]
#[symbol_key("StopAuth")]
#[data_storage_type(Instance)]
#[extend_ttl_info_instance]
pub struct StopAuthority(pub Address);

impl StopAuthority {
    pub fn require_stop_authority_auth(&self) {
        self.0.require_auth();
    }

    #[inline]
    pub fn require_exist_auth(env: &Env) -> Result<(), Error> {
        Self::get(env)?.0.require_auth();

        Ok(())
    }

    #[inline]
    pub fn as_address(&self) -> Address {
        self.0.clone()
    }
}
