#![no_std]

mod pool;
mod token;
mod unit_tests;

use generic_pool::generate_pool_contract;

use crate::pool::ThreePool;
use crate::token::ThreeToken;

generate_pool_contract!(TwoPoolContract, ThreePool, ThreeToken, 3);
