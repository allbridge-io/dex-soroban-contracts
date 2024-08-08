use test_case::test_case;

use crate::{
    utils::{Snapshot, TestingEnv, TestingEnvConfig},
    contracts::pool::Token as PoolToken
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
        &testing_env.token_a,
        &testing_env.token_b,
    );
}

#[test_case(1_000.0, 995.5, PoolToken::A, PoolToken::B, 998.986_014, 0.999_986; "base a2b")]
#[test_case(1_000.0, 995.5, PoolToken::B, PoolToken::A, 998.986_014, 0.999_986; "base b2a")]
#[test_case(1_000.0, 995.5, PoolToken::A, PoolToken::C, 998.986_014, 0.999_986; "base a2c")]
#[test_case(1_000.0, 995.5, PoolToken::C, PoolToken::A, 998.986_014, 0.999_986; "base c2a")]
#[test_case(1_000.0, 995.5, PoolToken::B, PoolToken::C, 998.986_014, 0.999_986; "base b2c")]
#[test_case(1_000.0, 995.5, PoolToken::C, PoolToken::B, 998.986_014, 0.999_986; "base c2b")]
#[test_case(0.001, 0.000_999, PoolToken::A, PoolToken::B, 0.000_999, 0.000_001; "smallest_swap")]
#[test_case(0.001, 0.0, PoolToken::B, PoolToken::A, 0.000_999, 0.000_001; "smallest_swap_b2a")]
fn simple_swaps(
    amount: f64,
    receive_amount_min: f64,
    token_from: PoolToken,
    token_to: PoolToken,
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
        testing_env.get_token(token_from),
        testing_env.get_token(token_to),
        expected_receive_amount,
        expected_fee,
    );
}

#[test_case(
    DepositArgs { amounts: (250_000.0, 0.0, 0.0), min_lp: 249_000.0 }, 10_000.0, 995.0, PoolToken::A, PoolToken::B, 9_966.249_774, 9.976_226; "swap_more_a"
)]
#[test_case(
    DepositArgs { amounts: (0.0, 250_000.0, 0.0), min_lp: 249_000.0 }, 10_000.0, 10010.0, PoolToken::A, PoolToken::B, 10_011.687_291, 10.021_709; "swap_more_b"
)]
#[test_case(
    DepositArgs { amounts: (0.0, 0.0, 250_000.0), min_lp: 249_000.0 }, 10_000.0, 10010.0, PoolToken::A, PoolToken::C, 10_011.687_291, 10.021_709; "swap_more_c_a2c"
)]
#[test_case(
    DepositArgs { amounts: (0.0, 0.0, 250_000.0), min_lp: 249_000.0 }, 10_000.0, 995.0, PoolToken::C, PoolToken::A, 9_966.249_774, 9.976_226; "swap_more_c_c2a"
)]
#[test_case(
    DepositArgs { amounts: (0.0, 0.0, 250_000.0), min_lp: 249_000.0 }, 10_000.0, 995.0, PoolToken::A, PoolToken::B, 9_988.639_362, 9.998_638; "swap_more_c_a2b"
)]
fn swap_disbalance(
    deposit_args: DepositArgs,
    amount: f64,
    receive_amount_min: f64,
    token_from: PoolToken,
    token_to: PoolToken,
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
        testing_env.get_token(token_from),
        testing_env.get_token(token_to),
        expected_receive_amount,
        expected_fee,
    );
}

#[test_case(PoolToken::A, PoolToken::B; "swap_more_than_pool_balance")]
#[test_case(PoolToken::B, PoolToken::A; "swap_more_than_pool_balance_b2a")]
#[test_case(PoolToken::A, PoolToken::C; "swap_more_than_pool_balance_a2C")]
#[test_case(PoolToken::B, PoolToken::C; "swap_more_than_pool_balance_b2c")]
#[test_case(PoolToken::C, PoolToken::A; "swap_more_than_pool_balance_c2a")]
#[test_case(PoolToken::C, PoolToken::B; "swap_more_than_pool_balance_c2b")]
fn swap_more_than_pool_balance(token_from: PoolToken, token_to: PoolToken) {
    let testing_env =
        TestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(500_000.0));
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let amount = 1_000_000.0;
    let deposit = (500_000.0, 500_000.0, 500_000.0);

    let snapshot_before = Snapshot::take(&testing_env);

    pool.deposit(alice, deposit, 1_000_000.0);
    pool.swap(alice, alice, amount, 500_000.0, testing_env.get_token(token_from), testing_env.get_token(token_to));
    // Bring pool back to balance by Alice
    pool.swap(alice, alice, amount, 500_000.0, testing_env.get_token(token_to), testing_env.get_token(token_from));
    pool.withdraw(alice, pool.user_lp_amount_f64(alice));

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw all");

    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);
    let alice_balance_after = snapshot_after.get_user_balances_sum(alice);

    assert!(alice_balance_after <= alice_balance_before);
}
