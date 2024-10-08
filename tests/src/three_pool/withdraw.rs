use test_case::test_case;

use crate::{
    three_pool_utils::{assert_rel_eq, float_to_uint, Snapshot, TestingEnv, TestingEnvConfig, TRIPLE_ZERO},
};

use super::{DepositArgs, DoWithdrawArgs};

#[test]
#[should_panic = "DexContract(NotEnoughAmount)"]
fn withdraw_full_and_try_again() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    pool.deposit(alice, (4_000.0, 5_000.0, 6_000.0), 8_999.0);
    pool.withdraw(alice, pool.user_lp_amount_f64(alice));
    pool.withdraw(alice, 0.001);
}

#[test]
#[should_panic = "DexContract(ZeroChanges)"]
fn withdraw_zero_change() {
    let testing_env = TestingEnv::default();
    testing_env.pool.withdraw(&testing_env.alice, 0.0);
}

#[test_case(
    TestingEnvConfig::default(),
    DepositArgs { amounts: (4_000.0, 5_000.0, 6_000.0), min_lp: 14_999.0 },
    DoWithdrawArgs { amount: 14_999.948, expected_amounts: (4_952.364, 4_999.984, 5_047.602), expected_fee: TRIPLE_ZERO, expected_rewards: TRIPLE_ZERO, expected_user_lp_diff: 14_999.948, expected_admin_fee: TRIPLE_ZERO }
    ; "base_withdraw"
)]
#[test_case(
    TestingEnvConfig::default().with_pool_fee_share(0.1).with_pool_admin_fee(20.0),
    DepositArgs { amounts: (4_000.0, 5_000.0, 6_000.0), min_lp: 14_999.0 },
    DoWithdrawArgs { amount: 14_999.948, expected_amounts: (4_947.411, 4_994.984, 5_042.554), expected_fee: (4.952_364, 4.999_984, 5.047_602), expected_rewards: TRIPLE_ZERO, expected_user_lp_diff: 14_999.948, expected_admin_fee: (0.990_472_8, 0.999_996_8, 1.009_520_4) }
    ; "withdraw_with_fee"
)]
#[test_case(
    TestingEnvConfig::default(),
    DepositArgs { amounts: (15_000.0, 25_000.0, 20_000.0), min_lp: 59_950.0 },
    DoWithdrawArgs { amount: 0.004, expected_amounts: (0.001, 0.001, 0.002), expected_fee: TRIPLE_ZERO, expected_rewards: TRIPLE_ZERO, expected_user_lp_diff: 0.004, expected_admin_fee: TRIPLE_ZERO }
    ; "smallest_withdraw"
)]
#[test_case(
    TestingEnvConfig::default().with_pool_fee_share(0.1),
    DepositArgs { amounts: (15_000.0, 25_000.0, 20_000.0), min_lp: 59_950.0 },
    DoWithdrawArgs { amount: 0.007, expected_amounts: (0.001, 0.001, 0.002), expected_fee: (0.000_002, 0.000_002, 0.000_003), expected_rewards: TRIPLE_ZERO, expected_user_lp_diff: 0.007, expected_admin_fee: TRIPLE_ZERO }
    ; "smallest_withdraw_with_fee"
)]
#[test_case(
    TestingEnvConfig::default(),
    DepositArgs { amounts: (50_000_000.0, 5_000.0, 5.0), min_lp: 21_358_206.68 },
    DoWithdrawArgs { amount: 21_358_206.68, expected_amounts: (49_406_036.726, 103_545.587, 98_619.774), expected_fee: TRIPLE_ZERO, expected_rewards: TRIPLE_ZERO, expected_user_lp_diff: 21_358_206.68, expected_admin_fee: TRIPLE_ZERO }
    ; "withdraw_disbalance"
)]
fn withdraw(config: TestingEnvConfig, deposit_args: DepositArgs, do_withdraw_args: DoWithdrawArgs) {
    let testing_env = TestingEnv::create(config);
    testing_env.pool.deposit(
        &testing_env.alice,
        deposit_args.amounts,
        deposit_args.min_lp,
    );
    testing_env.do_withdraw(
        &testing_env.alice,
        do_withdraw_args.amount,
        do_withdraw_args.expected_amounts,
        do_withdraw_args.expected_fee,
        do_withdraw_args.expected_rewards,
        do_withdraw_args.expected_user_lp_diff,
        do_withdraw_args.expected_admin_fee,
    );
}

#[test]
fn withdraw_with_rewards() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.1));
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ref token_a,
        ref token_b,
        ref token_c,
        ..
    } = testing_env;

    let deposits = (4_000.0, 5_000.0, 6_000.0);
    let expected_user_lp_diff = 14_999.948;
    // Alice has around 5% of the liquidity pool, we swap 1000 USD with 0.1% fee, which is 5% of 1 USD fee total
    let expected_rewards = (0.047_616_4, 0.047_618_8, 0.047_621_3);
    // Withdraw amounts sum is less than deposit amounts sum
    let expected_withdraw_amounts = (4_947.414, 4_994.984, 5_042.551);
    let expected_fee = (4.952_367, 4.999_984, 5.047_599);

    pool.deposit(alice, deposits, 14_999.0);
    pool.swap(bob, bob, 1_000.0, 998.0, token_a, token_b);
    pool.swap(bob, bob, 1_000.0, 998.0, token_b, token_c);
    pool.swap(bob, bob, 1_000.0, 998.0, token_c, token_a);

    testing_env.do_withdraw(
        alice,
        pool.user_lp_amount_f64(alice),
        expected_withdraw_amounts,
        expected_fee,
        expected_rewards,
        expected_user_lp_diff,
        TRIPLE_ZERO,
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

    let deposit = (4_000.0, 5_000.0, 6_000.0);
    let n = 4usize;

    pool.deposit(alice, deposit, 14_999.0);
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

// Alice large equal amounts (+100K each), pool is now 200K-200K, then swap by Bob 100K, then withdraw Alice all
// Alice should withdraw more than she deposited
#[test]
fn withdraw_alice_profit_and_bob_loss() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.1));
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ref token_a,
        ref token_b,
        ..
    } = testing_env;

    let deposit = (100_000.0, 100_000.0, 100_000.0);
    let swap_amount = 200_000.;
    let expected_user_withdraw_lp_diff = 300_000.0;
    let expected_rewards = (0.0, 94.999_999_9, 0.0);
    // Alice should withdraw more than she deposited
    let expected_withdraw_amounts = (199_800.0, 4_995.0, 99_900.0);
    let expected_alice_profit = 4_789.999_999_9;
    let expected_bob_losses = 10_190.0;
    let expected_fee = (200.0, 5.0, 100.0);

    pool.deposit(alice, deposit, 99_950.0);

    let snapshot_before_swap = Snapshot::take(&testing_env);
    pool.swap(bob, bob, swap_amount, 98336.0, token_a, token_b);
    let snapshot_after_swap = Snapshot::take(&testing_env);

    let (snapshot_before, snapshot_after) = testing_env.do_withdraw(
        alice,
        pool.user_lp_amount_f64(alice),
        expected_withdraw_amounts,
        expected_fee,
        expected_rewards,
        expected_user_withdraw_lp_diff,
        TRIPLE_ZERO,
    );

    let bob_b_diff =
        snapshot_after_swap.bob_b_balance - snapshot_before_swap.bob_b_balance;
    let bob_loss = float_to_uint(swap_amount, 7) - bob_b_diff;

    println!("{} {}", snapshot_after.get_user_balances_sum(alice), snapshot_before.get_user_balances_sum(alice));

    let alice_profit = snapshot_after.get_user_balances_sum(alice)
        - snapshot_before.get_user_balances_sum(alice)
        - float_to_uint(300_000.0, 7);

    assert!(alice_profit < bob_loss);
    assert_eq!(float_to_uint(expected_bob_losses, 7), bob_loss);
    assert_eq!(float_to_uint(expected_alice_profit, 7), alice_profit);
}

// Deposit Alice +200K in one token, pool is 300K to 100K, then Bob swap 100K to even out the pool (approx.), then Alice withdraw
// Alice should get less, Bob profit, should be less than Alice loss
#[test]
fn withdraw_alice_loss_and_bob_profit() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.1));
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ref token_a,
        ref token_b,
        ..
    } = testing_env;

    let deposit = (200_000.0, 0.0, 0.0);
    let swap_amount = 100_000.;
    let expected_user_withdraw_lp_diff = 199_501.791;
    let expected_rewards = (40.081_508_2, 0.0, 0.0);
    // Alice should withdraw less than she deposited (198_393.304)
    let expected_withdraw_amounts = (79_659.219, 79_800.429, 39_900.214);
    let expected_alice_loss = 600.056_491_8;
    let expected_bob_profit = 253.558_088;
    let expected_fee = (79.738_958, 79.880_310, 39.940_155);

    let snapshot_before_deposit = Snapshot::take(&testing_env);
    pool.deposit(alice, deposit, 198_000.0);

    let snapshot_before_swap = Snapshot::take(&testing_env);
    pool.swap(bob, bob, swap_amount, 100_000.0, token_b, token_a);
    let snapshot_after_swap = Snapshot::take(&testing_env);

    let (_, snapshot_after) = testing_env.do_withdraw(
        alice,
        pool.user_lp_amount_f64(alice),
        expected_withdraw_amounts,
        expected_fee,
        expected_rewards,
        expected_user_withdraw_lp_diff,
        TRIPLE_ZERO,
    );

    let bob_profit = snapshot_after_swap.get_user_balances_sum(bob)
        - snapshot_before_swap.get_user_balances_sum(bob);
    let alice_loss = snapshot_before_deposit.get_user_balances_sum(alice)
        - snapshot_after.get_user_balances_sum(alice);

    assert!(bob_profit < alice_loss);
    assert_rel_eq(float_to_uint(expected_bob_profit, 7), bob_profit, 1);
    assert_rel_eq(float_to_uint(expected_alice_loss, 7), dbg!(alice_loss), 1);
}
