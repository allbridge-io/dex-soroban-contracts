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

    // TODO: This is used later as LP, calculate LP instead of sum of deposits
    //      Or forget about it and use total user LP amount because we withdraw everything
    let deposit_sum = deposit.0 + deposit.1;

    pool.deposit(alice, deposit, 0.0).unwrap();

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let withdraw_amounts = pool.withdraw_amounts(alice);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.invariant_total_lp_less_or_equal_d();
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
    // TODO: Assert that alice LP is zero
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
    // TODO: WIthdraw minimum here
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
    // TODO: Balance here should be zero or near zero
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
    // TODO: Naming wthdraw_lp_amount
    let alice_lp_amount = 0.002;
    let withdraw_amounts = (0.001, 0.001);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.invariant_total_lp_less_or_equal_d();
    TestingEnvironment::assert_withdraw_event(&env, alice, alice_lp_amount, withdraw_amounts);
    TestingEnvironment::assert_claimed_reward_event(&env, alice, (0.0, 0.0));
    TestingEnvironment::assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        withdraw_amounts,
        (0.0, 0.0),
        // TODO: Use withdraw_lp_amount below
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

    // TODO: Hardcode, and remember, this is used to compare to LP, so this is actually expected LP diff
    //      name: expected_user_lp_diff
    let total_deposit = deposit.0 + deposit.1;
    let snapshot_before_deposit = Snapshot::take(&testing_env);

    pool.deposit(alice, deposit, 0.0).unwrap();

    // TODO: This is not expected LP diff, this is actual pool LP diff
    let expected_lp_amount = pool.get_lp_amount(snapshot_before_deposit.total_lp_amount);

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let withdraw_amounts = pool.withdraw_amounts(alice);

    // TODO: Assert withdraw amounts sum is less than deposit amounts sum

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.invariant_total_lp_less_or_equal_d();

    // TODO: This below should be assert equal of actual pool LP diff and expected user LP diff
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
    let testing_env =
        TestingEnvironment::create(&env, TestingEnvConfig::default().with_pool_fee_share(0.001));
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let deposits = (4000.0, 5000.0);
    // TODO: Hardcode and this is LP!!!, not deposits (expected_user_lp_diff)
    let total_deposits = deposits.0 + deposits.1;

    // Alice has around 5% of the liquidity pool, we swap 1000 USD with 0.1% fee, which is 5% of 1 USD fee total
    let expected_rewards = (0.0430620, 0.0430619);

    pool.deposit(alice, deposits, 0.0).unwrap();
    // TODO: Let Bob swap, keep Alice alone (and check everywhere else)
    pool.swap(alice, bob, 1000.0, 995.5, Direction::A2B)
        .unwrap();
    pool.swap(bob, alice, 1000.0, 999., Direction::B2A)
        .unwrap();

    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let withdraw_amounts = pool.withdraw_amounts(alice);

    // TODO: Assert withdraw amounts sum is less than deposit amounts sum

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    pool.invariant_total_lp_less_or_equal_d();
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

    // TODO: Use 0
    let alice_lp_amount = pool.user_lp_amount_f64(alice);
    let call_result = pool.withdraw(alice, alice_lp_amount);

    expect_contract_error(&env, call_result, shared::Error::ZeroChanges)
}

// TODO: Deposit Alice large equal amounts (+100K each), pool is now 200K-200K, then swap by Bob 100K, then withdraw Alice all
// Alice should withdraw more than she deposited, but use hardcode
// Check Alice profit to be less than Bob's loss

// TODO: And the oppisite, deposit Alice +200K in one token, pool is 300K to 100K, then Bob swap 100K to even out the pool (approx.), then Alice withdraw
// Alice should get less (hardcode), Bob profit, should be less than Alice loss