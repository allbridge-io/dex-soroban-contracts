use soroban_sdk::{contracttype, Address};

use crate::{
    events::{DepositEvent, RewardsClaimedEvent, WithdrawEvent},
    storage::sized_array::SizedU128Array,
};
use proc_macros::Event;

use super::utils::get_triple_tuple_from_sized_u128_array;

#[derive(Event)]
#[contracttype]
pub struct ThreeDeposit {
    pub user: Address,
    // system precision
    pub lp_amount: u128,
    // token precision
    pub amounts: (u128, u128, u128),
}

impl DepositEvent for ThreeDeposit {
    fn create(
        user: Address,
        lp_amount: u128,
        amounts: crate::storage::sized_array::SizedU128Array,
    ) -> Self {
        Self {
            user,
            lp_amount,
            amounts: get_triple_tuple_from_sized_u128_array(amounts),
        }
    }
}

#[derive(Event)]
#[contracttype]
pub struct ThreeWithdraw {
    pub user: Address,
    // system precision
    pub lp_amount: u128,
    // system precision
    pub amounts: (u128, u128, u128),
    // token precision
    pub fees: (u128, u128, u128),
}

impl WithdrawEvent for ThreeWithdraw {
    fn create(
        user: Address,
        lp_amount: u128,
        amounts: SizedU128Array,
        fees: SizedU128Array,
    ) -> Self {
        Self {
            user,
            lp_amount,
            amounts: get_triple_tuple_from_sized_u128_array(amounts),
            fees: get_triple_tuple_from_sized_u128_array(fees),
        }
    }
}

#[derive(Event)]
#[contracttype]
pub struct ThreeRewardsClaimed {
    pub user: Address,
    // token precision
    pub rewards: (u128, u128, u128),
}

impl RewardsClaimedEvent for ThreeRewardsClaimed {
    fn create(user: Address, rewards: SizedU128Array) -> Self {
        Self {
            user,
            rewards: get_triple_tuple_from_sized_u128_array(rewards),
        }
    }
}
