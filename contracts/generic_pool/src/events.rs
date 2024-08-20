use shared::Event;
use soroban_sdk::{contracttype, Address};

use crate::storage::sized_array::SizedU128Array;
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

pub trait DepositEvent: Event {
    fn create(user: Address, lp_amount: u128, amounts: SizedU128Array) -> Self;
}
pub trait WithdrawEvent: Event {
    fn create(
        user: Address,
        lp_amount: u128,
        amounts: SizedU128Array,
        fees: SizedU128Array,
    ) -> Self;
}
pub trait RewardsClaimedEvent: Event {
    fn create(user: Address, rewards: SizedU128Array) -> Self;
}
