use soroban_sdk::Env;

use crate::{
    contracts::pool::Direction,
    utils::{
        expect_auth_error, expect_contract_error, Snapshot, TestingEnvConfig, TestingEnvironment,
    },
};

#[test]
fn deposit_zero_amount() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let call_result = pool.deposit(alice, (0.0, 0.0), 0.0);
    expect_contract_error(&env, call_result, shared::Error::ZeroAmount)
}

#[test]
fn deposit_slippage() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let call_result = pool.deposit(alice, (100.0, 0.0), 1000.0);
    expect_contract_error(&env, call_result, shared::Error::Slippage)
}

#[test]
fn deposit() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposits = (100.0, 50.0);
    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, deposits, 0.0).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Deposit: 100 yusd, 50 yaro"));

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_deposit_event(&env, alice, deposits);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        deposits,
        (0.0, 0.0),
    );
}

#[test]
fn deposit_in_single_token() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposits = (100.0, 0.0);
    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, deposits, 0.0).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Deposit: 100 yusd, 50 yaro"));

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_deposit_event(&env, alice, deposits);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        deposits,
        (0.0, 0.0),
    );
}

#[test]
fn deposit_twice_in_different_tokens() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, (100.0, 0.0), 0.0).unwrap();
    pool.deposit(alice, (0.0, 100.0), 0.0).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Deposit: 100 yusd, 100 yaro"));

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        (100.0, 100.0),
        (0.0, 0.0),
    );
}

#[test]
fn withdraw() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    pool.deposit(alice, (4000.0, 5000.0), 0.0).unwrap();

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let withdraw_amounts = pool.withdraw_amounts(alice);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_withdraw_event(&env, alice, alice_lp_amount, withdraw_amounts);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        withdraw_amounts,
        (0.0, 0.0),
        9000.0,
    );
}

#[test]
fn withdraw_with_rewards() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default().with_pool_fee_share_bp(0.001),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let deposits = (4000.0, 5000.0);
    let total_deposits = deposits.0 + deposits.1;

    let expected_rewards = (0.0430620, 0.0430619);

    pool.deposit(alice, deposits, 0.0).unwrap();
    pool.swap(alice, bob, 1000.0, 995.5, Direction::A2B)
        .unwrap();
    pool.swap(bob, alice, 1000.0, 995.5, Direction::B2A)
        .unwrap();

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let withdraw_amounts = pool.withdraw_amounts(alice);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_withdraw_event(&env, alice, alice_lp_amount, withdraw_amounts);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, expected_rewards);
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        withdraw_amounts,
        expected_rewards,
        total_deposits,
    );
}

#[test]
fn withdraw_zero_change() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let call_result = pool.withdraw(alice, alice_lp_amount);

    expect_contract_error(&env, call_result, shared::Error::ZeroChanges)
}

#[test]
fn swap() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default()
            .with_pool_fee_share_bp(0.001)
            .with_yaro_admin_deposit(400_000.0)
            .with_yusd_admin_deposit(450_000.0),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 1000.0;
    let receive_amount_min = 995.5;

    pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B)
        .unwrap();

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Swap 1000 yusd => yaro"));

    pool.assert_total_lp_less_or_equal_d();
    testing_env.assert_swapped_event(&env, alice, alice, Direction::A2B, amount);

    TestingEnvironment::assert_swap(
        snapshot_before,
        snapshot_after,
        alice,
        alice,
        Direction::A2B,
        amount,
        receive_amount_min,
    );
}

#[test]
fn swap_insufficient_received_amount() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default().with_pool_fee_share_bp(0.001),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let amount = 1000.0;
    let receive_amount_min = 1000.5;

    let call_result = pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B);
    expect_contract_error(&env, call_result, shared::Error::InsufficientReceivedAmount)
}

#[test]
fn claim_admin_fee() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default()
            .with_pool_fee_share_bp(0.01)
            .with_pool_admin_fee(100),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let expected_admin_fees = (0.0099997, 0.0100002);

    pool.swap(alice, bob, 100.0, 98.0, Direction::B2A).unwrap();
    pool.swap(alice, bob, 100.0, 98.0, Direction::A2B).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_admin_fee().unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Admin claim fee"));

    TestingEnvironment::assert_claim_admin_fee(snapshot_before, snapshot_after, expected_admin_fees)
}

#[test]
fn claim_admin_fee_no_auth() {
    let env = Env::default();
    let testing_env =
        TestingEnvironment::create(&env, TestingEnvConfig::default().with_pool_admin_fee(100));
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    pool.swap(alice, bob, 100.0, 98.0, Direction::B2A).unwrap();
    pool.swap(alice, bob, 100.0, 98.0, Direction::A2B).unwrap();

    env.mock_auths(&[]);
    expect_auth_error(&env, pool.claim_admin_fee());
}

#[test]
fn claim_rewards() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default()
            .with_pool_fee_share_bp(0.01)
            .with_yaro_admin_deposit(0.0)
            .with_yusd_admin_deposit(0.0),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;
    let expected_rewards = (1.0012199, 0.9987799);

    pool.deposit(alice, (2000.0, 2000.0), 0.0).unwrap();
    pool.swap(alice, bob, 100.0, 98.0, Direction::A2B).unwrap();
    pool.swap(bob, alice, 100.0, 98.0, Direction::B2A).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_rewards(alice).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Alice claim rewards"));

    pool.assert_total_lp_less_or_equal_d();
    // TestingEnvironment::assert_claimed_reward_event(&env, alice, expected_rewards);
    TestingEnvironment::assert_claim(snapshot_before, snapshot_after, alice, expected_rewards);
}

#[test]
fn get_reward_after_second_deposit() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default()
            .with_pool_fee_share_bp(0.01)
            .with_yaro_admin_deposit(0.0)
            .with_yusd_admin_deposit(0.0),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let deposits = (2000.0, 2000.0);
    let expected_rewarsds = (1.0012199, 0.9987799);

    pool.deposit(alice, deposits, 0.0).unwrap();
    pool.swap(alice, bob, 100.0, 98.0, Direction::A2B).unwrap();
    pool.swap(bob, alice, 100.0, 98.0, Direction::B2A).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, deposits, 0.0).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, None);

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_deposit_event(&env, alice, deposits);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, expected_rewarsds);
    TestingEnvironment::assert_deposit(
        snapshot_before,
        snapshot_after,
        alice,
        deposits,
        expected_rewarsds,
    );
}
