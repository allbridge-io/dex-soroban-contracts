use soroban_sdk::Env;

use crate::{
    contracts::pool::Direction,
    utils::{expect_contract_error, Snapshot, TestingEnvConfig, TestingEnvironment},
};

#[test]
fn withdraw() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (4_000.0, 5_000.0);
    let deposit_sum = deposit.0 + deposit.1;

    pool.deposit(alice, deposit, 0.0).unwrap();

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let withdraw_amounts = pool.withdraw_amounts(alice);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.invariant_total_lp_less_or_equal_d().unwrap();
    TestingEnvironment::assert_withdraw_event(&env, alice, alice_lp_amount, withdraw_amounts);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        withdraw_amounts,
        (0.0, 0.0),
        deposit_sum,
    );
}

#[test]
fn withdraw_full_and_try_again() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (4_000.0, 5_000.0);

    pool.deposit(alice, deposit, 0.0).unwrap();
    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let call_result = pool.withdraw(alice, alice_lp_amount);
    expect_contract_error(&env, call_result, shared::Error::NotEnoughAmount);
}

#[test]
fn withdraw_multiply_times() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (4_000.0, 5_000.0);
    let n = 4usize;

    pool.deposit(alice, deposit, 0.0).unwrap();
    let total_alice_lp_amount = pool.user_lp_amount_f64(alice);
    let alice_lp_amount = total_alice_lp_amount / n as f64;
    let snapshot_before = Snapshot::take(&testing_env);
    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);

    for _ in 0..n {
        pool.withdraw(alice, alice_lp_amount).unwrap();
    }

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    let alice_balance_after = snapshot_before.get_user_balances_sum(alice);
    assert!(alice_balance_after <= alice_balance_before);
}

#[test]
fn smallest_withdraw() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    pool.deposit(alice, (15_000.0, 25_000.0), 0.0).unwrap();

    // 0.001 => ZeroChanges
    let alice_lp_amount = 0.002;
    let withdraw_amounts = (0.001, 0.001);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.invariant_total_lp_less_or_equal_d().unwrap();
    TestingEnvironment::assert_withdraw_event(&env, alice, alice_lp_amount, withdraw_amounts);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        withdraw_amounts,
        (0.0, 0.0),
        0.002,
    );
}

#[test]
fn withdraw_disbalance() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (50_000_000.0, 5_000.0);
    let total_deposit = deposit.0 + deposit.1;
    let snapshot_before_deposit = Snapshot::take(&testing_env);

    pool.deposit(alice, deposit, 0.0).unwrap();

    let expected_lp_amount = pool.get_lp_amount(snapshot_before_deposit.total_lp_amount);

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let withdraw_amounts = pool.withdraw_amounts(alice);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.invariant_total_lp_less_or_equal_d().unwrap();
    assert!(expected_lp_amount <= total_deposit);
    TestingEnvironment::assert_withdraw_event(&env, alice, alice_lp_amount, withdraw_amounts);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        withdraw_amounts,
        (0.0, 0.0),
        expected_lp_amount,
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

    pool.invariant_total_lp_less_or_equal_d().unwrap();
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
