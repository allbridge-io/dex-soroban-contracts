#![cfg(test)]

use test_case::test_case;

use crate::{
    contracts::pool::TwoToken as Token,
    contracts_wrappers::TestingEnvConfig,
    two_pool::{TwoPoolSnapshot, TwoPoolTestingEnv},
};

use super::DepositArgs;

#[test]
#[should_panic = "DexContract(InsufficientReceivedAmount)"]
fn swap_insufficient_received_amount() {
    let testing_env =
        TwoPoolTestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.1));
    testing_env.pool.swap(
        &testing_env.alice,
        &testing_env.alice,
        1000.0,
        1000.0,
        &testing_env.token_a,
        &testing_env.token_b,
    );
}

#[test_case(1_000.0, 995.5, Token::A, Token::B, 998.94006, 0.99994 ; "base")]
#[test_case(1000.0, 995.5, Token::B, Token::A, 998.94006, 0.99994 ; "base b2a")]
#[test_case(0.001, 0.000_999, Token::A, Token::B, 0.000_999, 0.000_001 ; "smallest_swap")]
#[test_case(0.001, 0.0, Token::B, Token::A, 0.000_999, 0.000_001 ; "smallest_swap_b2a")]
fn simple_swaps(
    amount: f64,
    receive_amount_min: f64,
    from_token: Token,
    to_token: Token,
    expected_receive_amount: f64,
    expected_fee: f64,
) {
    let testing_env = TwoPoolTestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.1)
            .with_admin_init_deposit(400_000.0),
    );

    testing_env.do_swap(
        &testing_env.alice,
        &testing_env.alice,
        amount,
        receive_amount_min,
        testing_env.get_token(from_token),
        testing_env.get_token(to_token),
        expected_receive_amount,
        expected_fee,
    );
}

#[test_case(DepositArgs { amounts: (0.0, 250_000.0), min_lp: 249_000.0 }, 10_000.0, 10090.0, Token::A, Token::B, 10_091.038_86, 10.101_140 ; "swap_more_yaro")]
#[test_case(DepositArgs { amounts: (250_000.0, 0.0), min_lp: 249_000.0 }, 10_000.0, 995.0, Token::A, Token::B, 9_880.313_796, 9.890_204 ; "swap_more_yusd")]
fn swap_disbalance(
    deposit_args: DepositArgs,
    amount: f64,
    receive_amount_min: f64,
    from_token: Token,
    to_token: Token,
    expected_receive_amount: f64,
    expected_fee: f64,
) {
    let testing_env = TwoPoolTestingEnv::create(
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
        testing_env.get_token(from_token),
        testing_env.get_token(to_token),
        expected_receive_amount,
        expected_fee,
    );
}

#[test_case(Token::A, Token::B ; "swap_more_than_pool_balance")]
#[test_case(Token::B, Token::A ; "swap_more_than_pool_balance_b2a")]
fn swap_more_than_pool_balance(from_token: Token, to_token: Token) {
    let testing_env =
        TwoPoolTestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(500_000.0));
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let amount = 1_000_000.0;
    let deposit = (500_000.0, 500_000.0);

    let snapshot_before = TwoPoolSnapshot::take(&testing_env);

    pool.deposit(alice, deposit, 1_000_000.0);
    pool.swap(
        alice,
        alice,
        amount,
        500_000.0,
        testing_env.get_token(from_token),
        testing_env.get_token(to_token),
    );
    // Bring pool back to balance by Alice
    pool.swap(
        alice,
        alice,
        amount,
        500_000.0,
        testing_env.get_token(to_token),
        testing_env.get_token(from_token),
    );
    pool.withdraw(alice, pool.user_lp_amount_f64(alice));

    let snapshot_after = TwoPoolSnapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw all");

    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);
    let alice_balance_after = snapshot_after.get_user_balances_sum(alice);

    assert!(alice_balance_after <= alice_balance_before);
}
