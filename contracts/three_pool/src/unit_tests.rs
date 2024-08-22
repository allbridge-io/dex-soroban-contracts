#![allow(clippy::inconsistent_digit_grouping)]
#![cfg(test)]

extern crate std;

use std::println;

use generic_pool::prelude::*;
use shared::{soroban_data::SimpleSorobanData, Error};
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

use crate::{pool::ThreePool, token::ThreeToken};

#[contract]
pub struct TestPool;

#[contractimpl]
impl TestPool {
    pub fn init(env: Env) {
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let token_c = Address::generate(&env);
        ThreePool::from_init_params(&env, 20, [token_a, token_b, token_c], [7, 7, 7], 100, 1)
            .save(&env);
    }

    pub fn set_balances(env: Env, new_balances: (u128, u128, u128)) -> Result<(), Error> {
        ThreePool::update(&env, |pool| {
            let (a, b, c) = new_balances;
            pool.token_balances = SizedU128Array::from_array(&env, [a, b, c]);
            pool.total_lp_amount = pool.get_current_d()?;
            Ok(())
        })
    }

    pub fn get_y(env: Env, x: u128, z: u128, d: u128) -> Result<u128, Error> {
        ThreePool::get(&env)?.get_y([x, z, d])
    }
    pub fn get_d(env: Env, x: u128, y: u128, z: u128) -> Result<u128, Error> {
        ThreePool::get(&env)?.get_d([x, y, z])
    }

    pub fn get_receive_amount(
        env: Env,
        amount: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<(u128, u128), Error> {
        let receive_amount =
            ThreePool::get(&env)?.get_receive_amount(amount, token_from, token_to)?;
        Ok((receive_amount.output, receive_amount.fee))
    }

    pub fn get_send_amount(
        env: Env,
        amount: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<(u128, u128), Error> {
        ThreePool::get(&env)?.get_send_amount(amount, token_from, token_to)
    }
}

#[test]
fn test_get_y() {
    let env = Env::default();

    let test_pool_id = env.register_contract(None, TestPool);
    let pool = TestPoolClient::new(&env, &test_pool_id);
    pool.init();

    assert_eq!(pool.get_y(&1_000_000, &1_000_000, &3_000_000), 1_000_000);

    let n = 100_000_000_000_000_000;
    let big_d = 157_831_140_060_220_325;
    let mid_d = 6_084_878_857_843_302;
    assert_eq!(pool.get_y(&n, &n, &(n * 3)), n);
    assert_eq!(pool.get_y(&n, &(n / 1_000), &big_d), n - 1);
    assert_eq!(pool.get_y(&n, &n, &big_d), n / 1_000 - 1);
    assert_eq!(pool.get_y(&n, &(n / 1_000), &mid_d), n / 1_000_000 - 1);
    assert_eq!(pool.get_y(&n, &(n / 1_000_000), &mid_d), n / 1_000 - 1);
    assert_eq!(pool.get_y(&(n / 1_000), &(n / 1_000_000), &mid_d), n - 14);
}

#[test]
fn test_get_d() {
    let env = Env::default();

    let test_pool_id = env.register_contract(None, TestPool);
    let pool = TestPoolClient::new(&env, &test_pool_id);
    pool.init();

    assert_eq!(pool.get_d(&2_000_000, &256_364, &5_000_000), 7_197_881);

    let n = 100_000_000_000_000_000;
    let big_d = 157_831_140_060_220_325;
    assert_eq!(pool.get_d(&n, &n, &n), n * 3);
    assert_eq!(pool.get_d(&n, &n, &(n / 1_000)), big_d);
    assert_eq!(
        pool.get_d(&n, &(n / 1_000), &(n / 1_000_000)),
        6_084_878_857_843_302
    );
}

#[test]
fn view_test() {
    let env = Env::default();

    let test_pool_id = env.register_contract(None, TestPool);
    let pool = TestPoolClient::new(&env, &test_pool_id);
    pool.init();
    pool.set_balances(&(200_000_000, 200_000_000, 200_000_000));

    let input = 10_000_0000000_u128;
    let (output, fee) = pool.get_receive_amount(&input, &ThreeToken::A, &ThreeToken::B);
    let (calc_input, calc_fee) = pool.get_send_amount(&output, &ThreeToken::A, &ThreeToken::B);

    println!("input: {}", input);
    println!("output: {}, fee: {}", output, fee);
    println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

    assert_eq!(input, calc_input);
    assert_eq!(fee, calc_fee);
}

#[test]
fn view_test_disbalance() {
    let env = Env::default();

    let test_pool_id = env.register_contract(None, TestPool);
    let pool = TestPoolClient::new(&env, &test_pool_id);
    pool.init();
    pool.set_balances(&(200_000_000, 500_000_000, 200_000_000));

    let input = 10_000_0000000_u128;
    let (output, fee) = pool.get_receive_amount(&input, &ThreeToken::A, &ThreeToken::B);
    let (calc_input, calc_fee) = pool.get_send_amount(&output, &ThreeToken::A, &ThreeToken::B);

    println!("input: {}", input);
    println!("output: {}, fee: {}", output, fee);
    println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

    assert_eq!(input, calc_input);
    assert_eq!(fee, calc_fee);
}
