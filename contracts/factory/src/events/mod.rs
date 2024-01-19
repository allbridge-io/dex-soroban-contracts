use soroban_sdk::{contracttype, Address};

use proc_macros::Event;

#[derive(Event)]
#[contracttype]
pub struct PairCreated {
    pub token0: Address,
    pub token1: Address,
    pub pool: Address,
}
