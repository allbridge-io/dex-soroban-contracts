use soroban_sdk::{contracttype, Address};

use proc_macros::Event;

#[derive(Event)]
#[contracttype]
pub struct Swapped {
    pub sender: Address,
    pub token: Address,
    pub to_amount: u128,
    pub from_amount: u128,
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
    pub rewards: (u128, u128),
}
