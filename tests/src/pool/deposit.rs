use crate::{
    contracts::pool::Direction,
    utils::{Snapshot, TestingEnv, TestingEnvConfig},
};

#[test]
#[should_panic = "DexContract(ZeroAmount)"]
fn deposit_zero_amount() {
    let testing_env = TestingEnv::default();
    testing_env
        .pool
        .deposit(&testing_env.alice, (0.0, 0.0), 0.0);
}

#[test]
#[should_panic = "DexContract(Slippage)"]
fn deposit_slippage() {
    let testing_env = TestingEnv::default();
    testing_env
        .pool
        .deposit(&testing_env.alice, (100.0, 0.0), 100.0);
}

#[test]
#[should_panic = "DexContract(PoolOverflow)"]
fn deposit_with_overflow() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ref yaro_token,
        ref yusd_token,
        ..
    } = testing_env;

    yusd_token.airdrop_amount(alice.as_ref(), 10_000_000_000.0);
    yaro_token.airdrop_amount(alice.as_ref(), 10_000_000_000.0);

    pool.deposit(alice, (600_000_000.0, 600_000_000.0), 0.0);
}

// TODO
#[test]
#[should_panic = "DexContract(InvalidFirstDeposit)"]
fn deposit_invalid_first_deposit() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(0.0));
    testing_env
        .pool
        .deposit(&testing_env.alice, (100.0, 25.0), 0.0);
}

#[test]
fn deposit() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposits = (100.0, 50.0);
    let expected_lp_amount = 150.0;

    let (snapshot_before, snapshot_after) =
        pool.deposit_with_snapshots(&testing_env, alice, deposits, 150.0);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 100 yusd, 50 yaro");

    testing_env.assert_deposit_event(alice, expected_lp_amount, deposits);
    testing_env.assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        deposits,
        (0.0, 0.0),
        expected_lp_amount,
    );
}

#[test]
fn deposit_disbalance() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (50_000_000.0, 5_000.0);
    let expected_lp_amount = 31_492_001.07;

    let (snapshot_before, snapshot_after) =
        pool.deposit_with_snapshots(&testing_env, alice, deposit, 0.0);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 50 000 000 yusd, 5 000 yaro");

    testing_env.assert_deposit_event(alice, expected_lp_amount, deposit);
    testing_env.assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        deposit,
        (0.0, 0.0),
        expected_lp_amount,
    );
}

// TODO: Also add test for depositing slightly less than MAX amount
//       Swap to big disbalance 1:100 and check for overflows

#[test]
fn smallest_deposit() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposits = (0.001, 0.001);
    let expected_lp_amount = 0.002;

    let (snapshot_before, snapshot_after) =
        pool.deposit_with_snapshots(&testing_env, alice, deposits, expected_lp_amount);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 0.001 yusd, 0.001 yaro");

    testing_env.assert_deposit_event(alice, expected_lp_amount, deposits);
    testing_env.assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        deposits,
        (0.0, 0.0),
        expected_lp_amount,
    );
}

#[test]
fn deposit_only_yusd() {
    let testing_env = TestingEnv::default();
    let deposits = (100.0, 0.0);
    let expected_lp_amount = 100.0;

    let (snapshot_before, snapshot_after) =
        testing_env
            .pool
            .deposit_with_snapshots(&testing_env, &testing_env.alice, deposits, 99.0);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 100 yusd");

    testing_env.assert_deposit_event(&testing_env.alice, expected_lp_amount, deposits);
    testing_env.assert_deposit(
        snapshot_before,
        snapshot_after,
        &testing_env.alice,
        deposits,
        (0.0, 0.0),
        expected_lp_amount,
    );
}

#[test]
fn deposit_only_yaro() {
    let testing_env = TestingEnv::default();
    let deposits = (0.0, 100.0);
    let expected_lp_amount = 100.0;

    let (snapshot_before, snapshot_after) =
        testing_env
            .pool
            .deposit_with_snapshots(&testing_env, &testing_env.alice, deposits, 99.0);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 100 yaro");

    testing_env.assert_deposit_event(&testing_env.alice, expected_lp_amount, deposits);
    testing_env.assert_deposit(
        snapshot_before,
        snapshot_after,
        &testing_env.alice,
        deposits,
        (0.0, 0.0),
        expected_lp_amount,
    );
}

#[test]
fn deposit_twice_in_different_tokens() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let expected_lp_amount = 200.0;

    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, (100.0, 0.0), 99.0);
    pool.deposit(alice, (0.0, 100.0), 99.0);
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Deposit: 100 yusd, 100 yaro");

    testing_env.assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        (100.0, 100.0),
        (0.0, 0.0),
        expected_lp_amount,
    );
}

#[test]
fn get_reward_after_second_deposit() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.01)
            .with_admin_init_deposit(0.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let deposits = (2000.0, 2000.0);
    let expected_rewards = (1.0012199, 0.9987799);
    let expected_lp_amount = 4000.0;

    pool.deposit(alice, deposits, 4000.0);
    pool.swap(alice, bob, 100.0, 98.0, Direction::A2B);
    pool.swap(bob, alice, 100.0, 99.0, Direction::B2A);

    let (snapshot_before, snapshot_after) =
        pool.deposit_with_snapshots(&testing_env, alice, deposits, 4000.0);
    snapshot_before.print_change_with(&snapshot_after, "After second deposit");

    testing_env.assert_deposit_event(alice, expected_lp_amount, deposits);
    testing_env.assert_claimed_reward_event(alice, expected_rewards);
    testing_env.assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        deposits,
        expected_rewards,
        expected_lp_amount,
    );
}
