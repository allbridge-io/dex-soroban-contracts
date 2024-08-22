#![allow(clippy::inconsistent_digit_grouping)]
#![cfg(test)]

extern crate std;
use std::println;

use generic_pool::prelude::*;
use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

use crate::{pool::TwoPool, token::TwoToken};

#[contract]
pub struct TestPool;

#[contractimpl]
impl TestPool {
    pub fn init(env: Env) {
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        TwoPool::from_init_params(&env, 20, [token_a, token_b], [7, 7], 100, 1).save(&env);
    }

    pub fn set_balances(env: Env, new_balances: (u128, u128)) -> Result<(), Error> {
        TwoPool::update(&env, |pool| {
            pool.token_balances =
                SizedU128Array::from_array(&env, [new_balances.0, new_balances.1]);
            pool.total_lp_amount = pool.get_current_d()?;
            Ok(())
        })
    }

    pub fn get_receive_amount(
        env: Env,
        amount: u128,
        token_from: TwoToken,
        token_to: TwoToken,
    ) -> Result<(u128, u128), Error> {
        let receive_amount =
            TwoPool::get(&env)?.get_receive_amount(amount, token_from, token_to)?;
        Ok((receive_amount.output, receive_amount.fee))
    }

    pub fn get_send_amount(
        env: Env,
        amount: u128,
        token_from: TwoToken,
        token_to: TwoToken,
    ) -> Result<(u128, u128), Error> {
        TwoPool::get(&env)?.get_send_amount(amount, token_from, token_to)
    }
}

#[test]
fn test() {
    let env = Env::default();

    let test_pool_id = env.register_contract(None, TestPool);
    let pool = TestPoolClient::new(&env, &test_pool_id);
    pool.init();
    pool.set_balances(&(200_000_000, 200_000_000));

    let input = 10_000_0000000_u128;
    let (output, fee) = pool.get_receive_amount(&input, &TwoToken::A, &TwoToken::B);
    let (calc_input, calc_fee) = pool.get_send_amount(&output, &TwoToken::A, &TwoToken::B);

    println!("input: {}", input);
    println!("output: {}, fee: {}", output, fee);
    println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

    assert_eq!(input, calc_input);
    assert_eq!(fee, calc_fee);
}

#[test]
fn test_disbalance() {
    let env = Env::default();

    let test_pool_id = env.register_contract(None, TestPool);
    let pool = TestPoolClient::new(&env, &test_pool_id);
    pool.init();
    pool.set_balances(&(200_000_000, 500_000_000));

    let input = 10_000_0000000_u128;
    let (output, fee) = pool.get_receive_amount(&input, &TwoToken::A, &TwoToken::B);
    let (calc_input, calc_fee) = pool.get_send_amount(&output, &TwoToken::A, &TwoToken::B);

    println!("input: {}", input);
    println!("output: {}, fee: {}", output, fee);
    println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

    assert_eq!(input, calc_input);
    assert_eq!(fee, calc_fee);
}
