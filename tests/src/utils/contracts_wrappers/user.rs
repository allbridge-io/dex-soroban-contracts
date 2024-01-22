use crate::utils::contract_id;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

pub struct User {
    pub address: Address,
}

impl User {
    pub fn generate(env: &Env) -> User {
        User {
            address: Address::generate(env),
        }
    }

    pub fn as_address(&self) -> Address {
        self.address.clone()
    }

    pub fn contract_id(&self) -> BytesN<32> {
        contract_id(&self.address)
    }
}
