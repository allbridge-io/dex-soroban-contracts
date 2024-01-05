use soroban_sdk::{contracttype, Address};

use proc_macros::Event;

#[derive(Event)]
#[contracttype]
pub struct SwappedFromVUsd {
    pub recipient: Address,
    pub token: Address,
    pub vusd_amount: u128,
    pub amount: u128,
    pub fee: u128,
}

#[derive(Event)]
#[contracttype]
pub struct SwappedToVUsd {
    pub sender: Address,
    pub token: Address,
    pub amount: u128,
    pub vusd_amount: u128,
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
    pub amount: u128,
}

#[derive(Event)]
#[contracttype]
pub struct RewardsClaimed {
    pub user: Address,
    pub amount: u128,
}

#[derive(Event)]
#[contracttype]
pub struct BalanceClaimed {
    pub user: Address,
    pub amount: u128,
}
