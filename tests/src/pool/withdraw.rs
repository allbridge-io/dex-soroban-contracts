use crate::{
    contracts::pool::Direction,
    utils::{assert_rel_eq, float_to_uint, Snapshot, TestingEnv, TestingEnvConfig},
};

#[test]
#[should_panic = "DexContract(NotEnoughAmount)"]
fn withdraw_full_and_try_again() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;
    let deposit = (4_000.0, 5_000.0);
    pool.deposit(alice, deposit, 8_999.0);
    pool.withdraw(alice, pool.user_lp_amount_f64(alice));
    pool.withdraw(alice, 0.001);
}

#[test]
#[should_panic = "DexContract(ZeroChanges)"]
fn withdraw_zero_change() {
    let testing_env = TestingEnv::default();
    testing_env.pool.withdraw(&testing_env.alice, 0.0);
}

#[test]
fn withdraw() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (4_000.0, 5_000.0);
    let expected_user_lp_diff = 8_999.942;
    let expected_withdraw_amounts = (4_478.441, 4_521.503);

    pool.deposit(alice, deposit, 8_999.0);

    let (snapshot_before, snapshot_after) =
        pool.withdraw_with_snapshots(&testing_env, alice, pool.user_lp_amount_f64(alice));
    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    assert_eq!(snapshot_after.alice_deposit.lp_amount, 0);
    testing_env.assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        expected_withdraw_amounts,
        (0.0, 0.0),
        expected_user_lp_diff,
    );
}

#[test]
fn withdraw_multiply_times() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (4_000.0, 5_000.0);
    let n = 4usize;

    pool.deposit(alice, deposit, 8_999.0);
    let total_alice_lp_amount = pool.user_lp_amount_f64(alice);
    let alice_lp_amount = total_alice_lp_amount / n as f64;
    let snapshot_before = Snapshot::take(&testing_env);
    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);

    for _ in 0..n {
        pool.withdraw(alice, alice_lp_amount);
    }

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    let alice_balance_after = snapshot_before.get_user_balances_sum(alice);

    assert_rel_eq(pool.user_deposit(alice).lp_amount, 0, 2);
    assert_eq!(alice_balance_after, alice_balance_before);
}

#[test]
fn smallest_withdraw() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    // 0.001 => ZeroChanges
    let withdraw_lp_amount = 0.002;
    let withdraw_amounts = (0.001, 0.001);

    pool.deposit(alice, (15_000.0, 25_000.0), 0.0);

    let (snapshot_before, snapshot_after) =
        pool.withdraw_with_snapshots(&testing_env, alice, withdraw_lp_amount);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    testing_env.assert_withdraw(
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
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let deposit = (50_000_000.0, 5_000.0);
    let expected_user_lp_diff = 31_492_001.072;
    let expected_withdraw_amounts = (49_783_831.892, 104_337.372);

    pool.deposit(alice, deposit, 0.0);

    // withdraw all
    let (snapshot_before, snapshot_after) =
        pool.withdraw_with_snapshots(&testing_env, alice, pool.user_lp_amount_f64(alice));
    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    testing_env.assert_withdraw(
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
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.001));
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let deposits = (4_000.0, 5_000.0);
    let expected_user_lp_diff = 8_999.942;
    // Alice has around 5% of the liquidity pool, we swap 1000 USD with 0.1% fee, which is 5% of 1 USD fee total
    let expected_rewards = (0.0430620, 0.0430619);
    // Withdraw amounts sum is less than deposit amounts sum (8999.944)
    let expected_withdraw_amounts = (4_478.441, 4_521.503);

    pool.deposit(alice, deposits, 0.0);
    pool.swap(bob, bob, 1_000.0, 995.5, Direction::A2B);
    pool.swap(bob, bob, 1_000.0, 999., Direction::B2A);

    // withdraw all
    let (snapshot_before, snapshot_after) =
        pool.withdraw_with_snapshots(&testing_env, alice, pool.user_lp_amount_f64(alice));
    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    testing_env.assert_claimed_reward_event(alice, expected_rewards);
    testing_env.assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        expected_withdraw_amounts,
        expected_rewards,
        expected_user_lp_diff,
    );
}

#[test]
fn withdraw_alice_profit() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.001));
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let deposit = (100_000.0, 100_000.0);
    let swap_amount = 100_000.;
    let expected_user_withdraw_lp_diff = 200_000.0;
    let expected_rewards = (0.0, 49.2179974);
    // Alice should withdraw more than she deposited (200_831.2209974)
    let expected_withdraw_amounts = (150_000.0, 50_782.003);
    let expected_alice_profit = 831.2209974;
    let expected_bob_losses = 1_662.4409950;

    pool.deposit(alice, deposit, 8999.0);

    let snapshot_before_swap = Snapshot::take(&testing_env);
    pool.swap(bob, bob, swap_amount, 98336.0, Direction::A2B);
    let snapshot_after_swap = Snapshot::take(&testing_env);

    let (snapshot_before, snapshot_after) =
        pool.withdraw_with_snapshots(&testing_env, alice, pool.user_lp_amount_f64(alice));
    snapshot_before.print_change_with(&snapshot_after, "Withdraw");

    let bob_yaro_diff =
        snapshot_after_swap.bob_yaro_balance - snapshot_before_swap.bob_yaro_balance;
    let bob_loss = float_to_uint(swap_amount, 7) - bob_yaro_diff;

    assert_eq!(float_to_uint(expected_bob_losses, 7), bob_loss);

    let sum_before = snapshot_before.get_user_balances_sum(alice);
    let sum_after = snapshot_after.get_user_balances_sum(alice);
    let alice_profit = sum_after - sum_before - float_to_uint(200_000.0, 7);

    assert_eq!(float_to_uint(expected_alice_profit, 7), alice_profit);
    assert!(
        alice_profit < bob_loss,
        "Alice's profit should be less than Bob's loss"
    );

    testing_env.assert_withdraw(
        snapshot_before,
        snapshot_after,
        alice,
        expected_withdraw_amounts,
        expected_rewards,
        expected_user_withdraw_lp_diff,
    );
}

// TODO: And the opposite, deposit Alice +200K in one token, pool is 300K to 100K, then Bob swap 100K to even out the pool (approx.), then Alice withdraw
// Alice should get less (hardcode), Bob profit, should be less than Alice loss
