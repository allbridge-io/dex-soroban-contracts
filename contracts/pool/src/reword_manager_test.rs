extern crate std;

#[allow(unused_imports)]
use std::prelude::*;

use shared::soroban_data::SimpleSorobanData;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

pub fn assert_rel_eq(a: u128, b: u128, d: u128) {
    assert!(a.abs_diff(b) <= d);
}

use crate::{
    methods::view::get_pool,
    storage::{pool::Pool, user_deposit::UserDeposit},
};

const TOKEN_DECIMALS: u32 = 7;

pub fn float_to_int(amount: f64, decimals: u32) -> u128 {
    (amount as f64 * 10.0f64.powi(decimals as i32)) as u128
}

const P: u128 = 2u128.pow(Pool::P as u32);
const SP: u32 = 3;

#[contract]
pub struct TestPoolForRewards;

#[contractimpl]
impl TestPoolForRewards {
    pub fn init(env: Env) {
        let token_a = Address::generate(&env);
        let token_b = Address::generate(&env);
        let lp_token = Address::generate(&env);
        let pool = Pool::from_init_params(
            20,
            token_a,
            token_b,
            lp_token,
            100,
            1,
            2000,
            TOKEN_DECIMALS,
            TOKEN_DECIMALS,
        );
        pool.save(&env);
    }

    pub fn deposit(env: Env, sender: Address, amount_lp: u128) -> u128 {
        let mut pool = Pool::get(&env).unwrap();
        let mut user_deposit = UserDeposit::get(&env, sender.clone());
        let result = pool.deposit_lp(&mut user_deposit, amount_lp);

        pool.save(&env);
        user_deposit.save(&env, sender);

        result
    }

    pub fn withdraw(env: Env, sender: Address, amount_lp: u128) -> u128 {
        let mut pool = Pool::get(&env).unwrap();
        let mut user_deposit = UserDeposit::get(&env, sender.clone());
        let reward_amount = pool.withdraw_lp(&mut user_deposit, amount_lp);

        pool.save(&env);
        user_deposit.save(&env, sender);

        reward_amount
    }

    pub fn add_rewards(env: Env, reward_amount: u128) {
        let mut pool = Pool::get(&env).unwrap();

        pool.add_rewards(reward_amount);
        pool.save(&env);
    }

    pub fn claim_rewards(env: Env, sender: Address) -> u128 {
        let pool = Pool::get(&env).unwrap();
        let mut user_deposit = UserDeposit::get(&env, sender.clone());
        let result = pool.claim_rewards(&mut user_deposit).unwrap();

        pool.save(&env);
        user_deposit.save(&env, sender);

        result
    }

    pub fn get_pool(env: Env) -> Pool {
        get_pool(env).unwrap()
    }

    pub fn get_user_deposit(env: Env, user: Address) -> UserDeposit {
        UserDeposit::get(&env, user)
    }

    pub fn assert_user_deposit(env: Env, user: Address, lp_amount: u128, reward_debt: u128) {
        assert_eq!(
            UserDeposit {
                lp_amount,
                reward_debt
            },
            UserDeposit::get(&env, user)
        );
    }
}

#[test]
fn common_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let test_pool_id = env.register_contract(None, TestPoolForRewards);
    let reward_manager = TestPoolForRewardsClient::new(&env, &test_pool_id);
    reward_manager.init();

    // Alice added liquidity
    let alice_liquidity = float_to_int(100.0, SP);
    reward_manager.deposit(&alice, &alice_liquidity);

    assert_eq!(reward_manager.get_pool().total_lp_amount, alice_liquidity);
    reward_manager.assert_user_deposit(&alice, &alice_liquidity, &0);

    // Added rewards
    reward_manager.add_rewards(&float_to_int(100.0, SP));
    assert_eq!(
        reward_manager.get_pool().acc_reward_per_share_p,
        P * 1 * 80 / 100
    );

    // Bob added liquidity
    let bob_liquidity = float_to_int(100.0, SP);
    reward_manager.deposit(&bob, &bob_liquidity);

    assert_eq!(
        reward_manager.get_pool().total_lp_amount,
        alice_liquidity + bob_liquidity
    );
    let bob_deposit = reward_manager.get_user_deposit(&bob);

    assert_eq!(bob_deposit.lp_amount, bob_liquidity);
    assert_rel_eq(bob_deposit.reward_debt, bob_liquidity * 80 / 100, 5);

    // Added rewards
    reward_manager.add_rewards(&float_to_int(100.0, SP));

    // 1.5, -20% admin fee
    assert_rel_eq(
        reward_manager.get_pool().acc_reward_per_share_p,
        P * 15 / 10 * 80 / 100,
        2,
    );

    // Alice claimed rewards
    let alice_claimed_rewards_result = reward_manager.claim_rewards(&alice);
    let alice_deposit = reward_manager.get_user_deposit(&alice);
    // -20% admin fee
    assert_rel_eq(
        alice_claimed_rewards_result,
        float_to_int(150.0, SP) * 80 / 100,
        5,
    );
    // -20% admin fee
    assert_rel_eq(
        alice_deposit.reward_debt,
        float_to_int(150.0, SP) * 80 / 100,
        2,
    );
    assert_eq!(alice_deposit.lp_amount, float_to_int(100.0, SP));

    // Bob claimed rewards
    let bob_claimed_rewards_result = reward_manager.claim_rewards(&bob);
    let bob_deposit = reward_manager.get_user_deposit(&bob);
    // -20% admin fee
    assert_rel_eq(
        bob_claimed_rewards_result,
        float_to_int(50.0, SP) * 80 / 100,
        5,
    );
    // -20% admin fee
    assert_rel_eq(
        bob_deposit.reward_debt,
        float_to_int(150.0, SP) * 80 / 100,
        2,
    );
    assert_eq!(bob_deposit.lp_amount, float_to_int(100.0, SP));

    // Added rewards
    reward_manager.add_rewards(&float_to_int(100.0, SP));
    // 2, -20% admin fee
    assert_rel_eq(
        reward_manager.get_pool().acc_reward_per_share_p,
        P * 20 / 10 * 80 / 100,
        2,
    );

    // Added rewards
    reward_manager.add_rewards(&float_to_int(100.0, SP));
    // 2.5, -20% admin fee
    assert_rel_eq(
        reward_manager.get_pool().acc_reward_per_share_p,
        P * 25 / 10 * 80 / 100,
        2,
    );

    // Alice withdraw liquidity
    let alice_withdraw_result = reward_manager.withdraw(&alice, &float_to_int(100.0, SP));
    assert_rel_eq(
        alice_withdraw_result + alice_claimed_rewards_result,
        float_to_int(250.0, SP) * 80 / 100,
        2,
    );
    assert_rel_eq(alice_withdraw_result, 80_000, 100);
    reward_manager.assert_user_deposit(&alice, &0, &0);
    // Added rewards
    reward_manager.add_rewards(&float_to_int(100.0, SP));
    // 3.5, -20% admin fee
    assert_rel_eq(
        reward_manager.get_pool().acc_reward_per_share_p,
        P * 35 / 10 * 80 / 100,
        2,
    );

    // Bob withdraw half liquidity
    let bob_withdraw_result = reward_manager.withdraw(&bob, &float_to_int(50.0, SP));
    let bob_deposit = reward_manager.get_user_deposit(&bob);
    assert_rel_eq(
        bob_withdraw_result + bob_claimed_rewards_result,
        float_to_int(250.0, SP) * 80 / 100,
        20_000,
    );
    // -20% admin fee
    assert_rel_eq(
        bob_deposit.reward_debt,
        float_to_int(175.0, SP) * 80 / 100,
        2,
    );
    assert_eq!(bob_deposit.lp_amount, float_to_int(50.0, SP));
    assert_eq!(
        reward_manager.get_pool().admin_fee_amount,
        float_to_int(100.0, SP) * 5 * 20 / 100
    );
}
