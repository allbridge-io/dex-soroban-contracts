use crate::utils::contract_id;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

pub struct User {
    pub address: Address,
    pub tag: &'static str,
}

impl AsRef<Address> for User {
    fn as_ref(&self) -> &Address {
        &self.address
    }
}

impl User {
    pub fn generate(env: &Env, tag: &'static str) -> User {
        User {
            address: Address::generate(env),
            tag,
        }
    }

    pub fn as_address(&self) -> Address {
        self.address.clone()
    }

    pub fn contract_id(&self) -> BytesN<32> {
        contract_id(&self.address)
    }
}
