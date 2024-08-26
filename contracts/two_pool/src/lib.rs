#![no_std]

mod pool;
mod token;
mod unit_tests;

use generic_pool::generate_pool_contract;

use crate::pool::TwoPool;
use crate::token::TwoToken;

generate_pool_contract!(TwoPoolContract, TwoPool, TwoToken, 2);
