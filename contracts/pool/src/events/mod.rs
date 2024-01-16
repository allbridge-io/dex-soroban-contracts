use soroban_sdk::{contracttype, Address};

use proc_macros::Event;

use crate::storage::double_value::DoubleValue;

#[derive(Event)]
#[contracttype]
pub struct Swapped {
    pub sender: Address,
    pub recipient: Address,
    pub from_token: Address,
    pub to_token: Address,
    pub from_amount: u128,
    pub to_amount: u128,
    pub fee: u128,
}

#[derive(Event)]
#[contracttype]
pub struct Deposit {
    pub user: Address,
    pub amount: u128,
}

#[derive(Event)]
#[contracttype]
pub struct Withdraw {
    pub user: Address,
    pub lp_amount: u128,
}

#[derive(Event)]
#[contracttype]
pub struct RewardsClaimed {
    pub user: Address,
    pub rewards: DoubleValue,
}
