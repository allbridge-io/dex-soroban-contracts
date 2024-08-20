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

pub mod three_pool_events {
    use soroban_sdk::{contracttype, Address};

    use proc_macros::Event;

    use crate::storage::sized_array::SizedU128Array;

    use super::{DepositEvent, RewardsClaimedEvent, WithdrawEvent};

    #[derive(Event)]
    #[contracttype]
    pub struct Deposit {
        pub user: Address,
        // system precision
        pub lp_amount: u128,
        // token precision
        pub amounts: (u128, u128, u128),
    }

    fn get_tuple_from_sized_u128_array(array: SizedU128Array) -> (u128, u128, u128) {
        (array.get(0usize), array.get(1usize), array.get(2usize))
    }

    impl DepositEvent for Deposit {
        fn create(
            user: Address,
            lp_amount: u128,
            amounts: crate::storage::sized_array::SizedU128Array,
        ) -> Self {
            Self {
                user,
                lp_amount,
                amounts: get_tuple_from_sized_u128_array(amounts),
            }
        }
    }

    #[derive(Event)]
    #[contracttype]
    pub struct Withdraw {
        pub user: Address,
        // system precision
        pub lp_amount: u128,
        // system precision
        pub amounts: (u128, u128, u128),
        // token precision
        pub fees: (u128, u128, u128),
    }

    impl WithdrawEvent for Withdraw {
        fn create(
            user: Address,
            lp_amount: u128,
            amounts: SizedU128Array,
            fees: SizedU128Array,
        ) -> Self {
            Self {
                user,
                lp_amount,
                amounts: get_tuple_from_sized_u128_array(amounts),
                fees: get_tuple_from_sized_u128_array(fees),
            }
        }
    }

    #[derive(Event)]
    #[contracttype]
    pub struct RewardsClaimed {
        pub user: Address,
        // token precision
        pub rewards: (u128, u128, u128),
    }

    impl RewardsClaimedEvent for RewardsClaimed {
        fn create(user: Address, rewards: SizedU128Array) -> Self {
            Self {
                user,
                rewards: get_tuple_from_sized_u128_array(rewards),
            }
        }
    }
}
