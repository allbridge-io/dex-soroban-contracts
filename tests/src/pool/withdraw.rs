use soroban_sdk::Env;

use crate::{
    contracts::pool::Direction,
    utils::{assert_rel_eq, expect_contract_error, Snapshot, TestingEnvConfig, TestingEnvironment},
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
    let expected_user_lp_diff = 8999.942;
    let expected_withdraw_amounts = (4478.441, 4521.503);

    pool.deposit(alice, deposit, 8999.0).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, pool.user_lp_amount_f64(alice))
        .unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    assert_eq!(snapshot_after.alice_deposit.lp_amount, 0);
    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_withdraw_event(
        &env,
        alice,
        expected_user_lp_diff,
        expected_withdraw_amounts,
    );
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        expected_withdraw_amounts,
        (0.0, 0.0),
        expected_user_lp_diff,
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

    pool.deposit(alice, deposit, 8999.0).unwrap();
    pool.withdraw(alice, pool.user_lp_amount_f64(alice))
        .unwrap();

    let call_result = pool.withdraw(alice, 0.001);
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

    pool.deposit(alice, deposit, 8999.0).unwrap();
    let total_alice_lp_amount = pool.user_lp_amount_f64(alice);
    let alice_lp_amount = total_alice_lp_amount / n as f64;
    let snapshot_before = Snapshot::take(&testing_env);
    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);

    for _ in 0..n {
        pool.withdraw(alice, alice_lp_amount).unwrap();
    }

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    let alice_balance_after = snapshot_before.get_user_balances_sum(alice);

    assert_rel_eq(pool.user_deposit(alice).lp_amount, 0, 2);
    assert_eq!(alice_balance_after, alice_balance_before);
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

    // 0.001 => ZeroChanges
    let withdraw_lp_amount = 0.002;
    let withdraw_amounts = (0.001, 0.001);

    pool.deposit(alice, (15_000.0, 25_000.0), 0.0).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, withdraw_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_withdraw_event(&env, alice, withdraw_lp_amount, withdraw_amounts);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        withdraw_amounts,
        (0.0, 0.0),
        withdraw_lp_amount,
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
    let expected_user_lp_diff = 31_492_001.072;
    let expected_withdraw_amounts = (49783831.892, 104337.372);

    pool.deposit(alice, deposit, 0.0).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    // withdraw all
    pool.withdraw(alice, pool.user_lp_amount_f64(alice))
        .unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_withdraw_event(
        &env,
        alice,
        expected_user_lp_diff,
        expected_withdraw_amounts,
    );
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        expected_withdraw_amounts,
        (0.0, 0.0),
        expected_user_lp_diff,
    );
}

#[test]
fn withdraw_with_rewards() {
    let env = Env::default();
    let testing_env =
        TestingEnvironment::create(&env, TestingEnvConfig::default().with_pool_fee_share(0.001));
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let deposits = (4000.0, 5000.0);
    let expected_user_lp_diff = 8999.942;
    // Alice has around 5% of the liquidity pool, we swap 1000 USD with 0.1% fee, which is 5% of 1 USD fee total
    let expected_rewards = (0.0430620, 0.0430619);
    let expected_withdraw_amounts = (4478.441, 4521.503);

    pool.deposit(alice, deposits, 0.0).unwrap();
    pool.swap(bob, bob, 1000.0, 995.5, Direction::A2B).unwrap();
    pool.swap(bob, bob, 1000.0, 999., Direction::B2A).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    // withdraw all
    pool.withdraw(alice, pool.user_lp_amount_f64(alice))
        .unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_withdraw_event(
        &env,
        alice,
        expected_user_lp_diff,
        expected_withdraw_amounts,
    );
    TestingEnvironment::assert_claimed_reward_event(&env, alice, expected_rewards);
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        expected_withdraw_amounts,
        expected_rewards,
        expected_user_lp_diff,
    );
}

#[test]
fn withdraw_zero_change() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let call_result = testing_env.pool.withdraw(&testing_env.alice, 0.0);

    expect_contract_error(&env, call_result, shared::Error::ZeroChanges)
}

// TODO: Deposit Alice large equal amounts (+100K each), pool is now 200K-200K, then swap by Bob 100K, then withdraw Alice all
// Alice should withdraw more than she deposited, but use hardcode
// Check Alice profit to be less than Bob's loss

// TODO: And the opposite, deposit Alice +200K in one token, pool is 300K to 100K, then Bob swap 100K to even out the pool (approx.), then Alice withdraw
// Alice should get less (hardcode), Bob profit, should be less than Alice loss
