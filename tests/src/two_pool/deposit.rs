#![cfg(test)]

use test_case::test_case;

use crate::two_pool::{TwoPoolSnapshot, TwoPoolTestingEnv};
use crate::utils::{PoolClient, Snapshot, TestingEnv, TestingEnvConfig};

#[test]
#[should_panic = "DexContract(ZeroAmount)"]
fn deposit_zero_amount() {
    let testing_env = TwoPoolTestingEnv::default();
    testing_env
        .pool
        .deposit(&testing_env.alice, [0.0, 0.0], 0.0);
}

#[test]
#[should_panic = "DexContract(Slippage)"]
fn deposit_slippage() {
    let testing_env = TwoPoolTestingEnv::default();
    testing_env
        .pool
        .deposit(&testing_env.alice, [100.0, 0.0], 100.0);
}

#[test]
#[should_panic = "DexContract(PoolOverflow)"]
fn deposit_with_overflow() {
    let testing_env = TwoPoolTestingEnv::default();
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        token_b: ref yaro_token,
        token_a: ref yusd_token,
        ..
    } = testing_env;

    yusd_token.airdrop(alice, 10_000_000_000.0);
    yaro_token.airdrop(alice, 10_000_000_000.0);

    pool.deposit(alice, [600_000_000.0, 600_000_000.0], 0.0);
}

#[test]
#[should_panic = "DexContract(InvalidFirstDeposit)"]
fn deposit_invalid_first_deposit() {
    let testing_env =
        TwoPoolTestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(0.0));
    testing_env
        .pool
        .deposit(&testing_env.alice, [100.0, 25.0], 0.0);
}

#[test_case([100.0, 50.0], [0.0;2], 150.0 ; "base")]
#[test_case([50_000_000.0, 5_000.0], [0.0;2], 31_492_001.072 ; "deposit_disbalance")]
#[test_case([0.001, 0.001], [0.0;2], 0.002 ; "smallest_deposit")]
#[test_case([100.0, 0.0], [0.0;2], 99.998 ; "deposit_only_yusd")]
#[test_case([0.0, 100.0], [0.0;2], 99.998 ; "deposit_only_yaro")]
fn deposit(deposit: [f64; 2], expected_rewards: [f64; 2], expected_lp: f64) {
    let testing_env = TwoPoolTestingEnv::default();
    testing_env.do_deposit(&testing_env.alice, deposit, expected_rewards, expected_lp);
}

#[test]
fn deposit_twice_in_different_tokens() {
    let testing_env = TwoPoolTestingEnv::default();
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let expected_lp_amount = 200.0;

    let snapshot_before = TwoPoolSnapshot::take(&testing_env);
    pool.deposit(alice, [100.0, 0.0], 99.0);
    pool.deposit(alice, [0.0, 100.0], 99.0);
    let snapshot_after = TwoPoolSnapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 100 yusd, 100 yaro");

    testing_env.assert_deposit_without_event(
        snapshot_before,
        snapshot_after,
        alice,
        [100.0, 100.0],
        [0.0; 2],
        expected_lp_amount,
    );
}

#[test]
fn get_reward_after_second_deposit() {
    let testing_env = TwoPoolTestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_admin_init_deposit(0.0),
    );
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        ref bob,
        token_a: ref yusd_token,
        token_b: ref yaro_token,
        ..
    } = testing_env;

    let deposit = [2_000.0, 2_000.0];
    let expected_rewards = [1.001_219_9, 0.998_779_9];
    let expected_lp_amount = 4_000.0;

    pool.deposit(alice, deposit, 4_000.0);
    pool.swap(alice, bob, 100.0, 98.0, yusd_token, yaro_token);
    pool.swap(bob, alice, 100.0, 99.0, yaro_token, yusd_token);

    testing_env.do_deposit(
        &testing_env.alice,
        deposit,
        expected_rewards,
        expected_lp_amount,
    );
}
