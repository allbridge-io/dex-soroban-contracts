use test_case::test_case;

use crate::{
    three_pool_utils::{Snapshot, TestingEnv, TestingEnvConfig, TRIPLE_ZERO},
};

#[test]
#[should_panic = "DexContract(ZeroAmount)"]
fn deposit_zero_amount() {
    let testing_env = TestingEnv::default();
    testing_env
        .pool
        .deposit(&testing_env.alice, (0.0, 0.0, 0.0), 0.0);
}

#[test]
#[should_panic = "DexContract(Slippage)"]
fn deposit_slippage() {
    let testing_env = TestingEnv::default();
    testing_env
        .pool
        .deposit(&testing_env.alice, (100.0, 0.0, 0.0), 100.1);
}

#[test]
#[should_panic = "DexContract(PoolOverflow)"]
fn deposit_with_overflow() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        token_b: ref b_token,
        token_a: ref a_token,
        ..
    } = testing_env;

    a_token.airdrop(alice, 10_000_000_000.0);
    b_token.airdrop(alice, 10_000_000_000.0);

    pool.deposit(alice, (600_000_000.0, 600_000_000.0, 600_000_000.0), 0.0);
}


#[should_panic = "DexContract(InvalidFirstDeposit)"]
#[test_case((99.0, 100.0, 100.0); "invalid_a")]
#[test_case((100.0, 99.0, 100.0); "invalid_b")]
#[test_case((100.0, 100.0, 99.0); "invalid_c")]
fn deposit_invalid_first_deposit(deposit: (f64, f64, f64)) {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(0.0));
    testing_env
        .pool
        .deposit(&testing_env.alice, deposit, 0.0);
}

#[test_case((100.0, 50.0, 75.0), TRIPLE_ZERO, 225.0 ; "base")]
#[test_case((50_000_000.0, 5_000.0, 5.0), TRIPLE_ZERO, 21_358_206.68 ; "deposit_disbalance")]
#[test_case((0.001, 0.001, 0.0), TRIPLE_ZERO, 0.002 ; "smallest_deposit")]
#[test_case((100.0, 0.0, 0.0), TRIPLE_ZERO, 100.0 ; "deposit_only_a")]
#[test_case((0.0, 100.0, 0.0), TRIPLE_ZERO, 100.0 ; "deposit_only_b")]
#[test_case((0.0, 0.0, 100.0), TRIPLE_ZERO, 100.0 ; "deposit_only_c")]
fn deposit(deposit: (f64, f64, f64), expected_rewards: (f64, f64, f64), expected_lp: f64) {
    let testing_env = TestingEnv::default();
    testing_env.do_deposit(&testing_env.alice, deposit, expected_rewards, expected_lp);
}

#[test]
fn deposit_three_times_in_different_tokens() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let expected_lp_amount = 300.0;

    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, (100.0, 0.0, 0.0), 99.0);
    pool.deposit(alice, (0.0, 100.0, 0.0), 99.0);
    pool.deposit(alice, (0.0, 0.0, 100.0), 99.0);
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 100 a, 100 b, 100 c");

    testing_env.assert_deposit_without_event(
        snapshot_before,
        snapshot_after,
        alice,
        (100.0, 100.0, 100.0),
        TRIPLE_ZERO,
        expected_lp_amount,
    );
}

#[test]
fn get_reward_after_third_deposit() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_admin_init_deposit(0.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ref token_a,
        ref token_b,
        ref token_c,
        ..
    } = testing_env;

    let deposit = (2_000.0, 2_000.0, 2_000.0);
    let expected_rewards = (1.000_269_9, 0.999_729_9, 0.999_999_9);
    let expected_lp_amount = 6_000.0;

    pool.deposit(alice, deposit, 4_000.0);
    pool.swap(alice, bob, 100.0, 98.0, token_a, token_b);
    pool.swap(bob, alice, 100.0, 99.0, token_b, token_c);
    pool.swap(bob, alice, 100.0, 99.0, token_c, token_a);

    testing_env.do_deposit(
        &testing_env.alice,
        deposit,
        expected_rewards,
        expected_lp_amount,
    );
}
