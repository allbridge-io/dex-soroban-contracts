use test_case::test_case;

use crate::{
    contracts::pool::Direction,
    utils::{Snapshot, TestingEnv, TestingEnvConfig},
};

use super::DepositArgs;

#[test]
#[should_panic = "DexContract(InsufficientReceivedAmount)"]
fn swap_insufficient_received_amount() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.1));
    testing_env.pool.swap(
        &testing_env.alice,
        &testing_env.alice,
        1000.0,
        1000.0,
        Direction::A2B,
    );
}

#[test_case(1_000.0, 995.5, Direction::A2B, 998.94006, 0.99994 ; "base")]
#[test_case(1000.0, 995.5, Direction::B2A, 998.94006, 0.99994 ; "base b2a")]
#[test_case(0.001, 0.000_999, Direction::A2B, 0.000_999, 0.000_001 ; "smallest_swap")]
#[test_case(0.001, 0.0, Direction::B2A, 0.000_999, 0.000_001 ; "smallest_swap_b2a")]
fn simple_swaps(
    amount: f64,
    receive_amount_min: f64,
    direction: Direction,
    expected_receive_amount: f64,
    expected_fee: f64,
) {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.1)
            .with_admin_init_deposit(400_000.0),
    );

    testing_env.do_swap(
        &testing_env.alice,
        &testing_env.alice,
        amount,
        receive_amount_min,
        direction,
        expected_receive_amount,
        expected_fee,
    );
}

#[test_case(DepositArgs { amounts: (0.0, 250_000.0), min_lp: 249_000.0 }, 10_000.0, 10090.0, Direction::A2B, 10_091.038_86, 10.101_140 ; "swap_more_yaro")]
#[test_case(DepositArgs { amounts: (250_000.0, 0.0), min_lp: 249_000.0 }, 10_000.0, 995.0, Direction::A2B, 9_880.313_796, 9.890_204 ; "swap_more_yusd")]
fn swap_disbalance(
    deposit_args: DepositArgs,
    amount: f64,
    receive_amount_min: f64,
    direction: Direction,
    expected_receive_amount: f64,
    expected_fee: f64,
) {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.1)
            .with_admin_init_deposit(500_000.0),
    );

    testing_env.pool.deposit(
        &testing_env.admin,
        deposit_args.amounts,
        deposit_args.min_lp,
    );

    testing_env.do_swap(
        &testing_env.alice,
        &testing_env.alice,
        amount,
        receive_amount_min,
        direction,
        expected_receive_amount,
        expected_fee,
    );
}

#[test_case(Direction::A2B ; "swap_more_than_pool_balance")]
#[test_case(Direction::B2A ; "swap_more_than_pool_balance_b2a")]
fn swap_more_than_pool_balance(direction: Direction) {
    let testing_env =
        TestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(500_000.0));
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let amount = 1_000_000.0;
    let deposit = (500_000.0, 500_000.0);

    let snapshot_before = Snapshot::take(&testing_env);

    pool.deposit(alice, deposit, 1_000_000.0);
    pool.swap(alice, alice, amount, 500_000.0, direction.clone());
    // Bring pool back to balance by Alice
    pool.swap(alice, alice, amount, 500_000.0, direction.reverse());
    pool.withdraw(alice, pool.user_lp_amount_f64(alice));

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw all");

    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);
    let alice_balance_after = snapshot_after.get_user_balances_sum(alice);

    assert!(alice_balance_after <= alice_balance_before);
}
