use soroban_sdk::{contracttype, Address, Vec};

use proc_macros::Event;

#[derive(Event)]
#[contracttype]
pub struct Swapped {
    pub sender: Address,
    pub recipient: Address,
    pub from_token: Address,
    pub to_token: Address,
    // token precision
    pub from_amount: u128,
    // token precision
    pub to_amount: u128,
    // token precision
    pub fee: u128,
}

#[derive(Event)]
#[contracttype]
pub struct Deposit {
    pub user: Address,
    // system precision
    pub lp_amount: u128,
    // token precision
    pub amounts: Vec<u128>,
}

#[derive(Event)]
#[contracttype]
pub struct Withdraw {
    pub user: Address,
    // system precision
    pub lp_amount: u128,
    // system precision
    pub amounts: Vec<u128>,
    // token precision
    pub fees: Vec<u128>,
}

#[derive(Event)]
#[contracttype]
pub struct RewardsClaimed {
    pub user: Address,
    // token precision
    pub rewards: Vec<u128>,
}
