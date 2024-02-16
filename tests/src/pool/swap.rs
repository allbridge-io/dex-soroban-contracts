use crate::{
    contracts::pool::Direction,
    utils::{uint_to_float, Snapshot, TestingEnv, TestingEnvConfig},
};

#[test]
fn swap() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.001)
            .with_admin_init_deposit(400_000.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 1000.0;
    // TODO: Also introduce receive_amount_max and check range
    let receive_amount_min = 995.5;
    let (expected_receive_amount, fee) = testing_env.pool.receive_amount(amount, Direction::A2B);
    let expected_receive_amount = uint_to_float(expected_receive_amount, 7);

    // TODO: Assert expected_receive_amount to be less that amount and more than receive_amount_min

    pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B);

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Swap 1000 yusd => yaro");

    testing_env.assert_swap(
        snapshot_before,
        snapshot_after,
        alice,
        alice,
        Direction::A2B,
        amount,
        receive_amount_min..=amount,
        expected_receive_amount,
        fee,
    );
}

#[test]
fn swap_b2a() {
    // TODO: See todos from swap()

    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.001)
            .with_admin_init_deposit(400_000.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 1000.0;
    let receive_amount_min = 995.5;
    let (expected_receive_amount, fee) = testing_env.pool.receive_amount(amount, Direction::B2A);
    let expected_receive_amount = uint_to_float(expected_receive_amount, 7);

    pool.swap(alice, alice, amount, receive_amount_min, Direction::B2A);

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Swap 1000 yusd => yaro");

    testing_env.assert_swap(
        snapshot_before,
        snapshot_after,
        alice,
        alice,
        Direction::B2A,
        amount,
        receive_amount_min..=amount,
        expected_receive_amount,
        fee,
    );
}

#[test]
fn smallest_swap() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.001)
            .with_admin_init_deposit(400_000.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 0.001;
    let receive_amount_min = 0.0;
    let (expected_receive_amount, fee) = testing_env.pool.receive_amount(amount, Direction::A2B);
    let expected_receive_amount = uint_to_float(expected_receive_amount, 7);

    // TODO: expected_receive_amount should be zero

    dbg!(expected_receive_amount);

    pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B);

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Swap 0.0000001 yusd => yaro");

    // TODO: Same stuff with ranges as before, but here the range is from 0 to 0
    testing_env.assert_swap(
        snapshot_before,
        snapshot_after,
        alice,
        alice,
        Direction::A2B,
        amount,
        receive_amount_min..=amount,
        expected_receive_amount,
        fee,
    );
}

#[test]
fn smallest_swap_b2a() {
    // TODO: See todos from smallest_swap()
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.001)
            .with_admin_init_deposit(400_000.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 0.001;
    let receive_amount_min = 0.0;
    let (expected_receive_amount, fee) = testing_env.pool.receive_amount(amount, Direction::B2A);
    let expected_receive_amount = uint_to_float(expected_receive_amount, 7);

    pool.swap(alice, alice, amount, receive_amount_min, Direction::B2A);

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Swap 0.0000001 yusd => yaro");

    testing_env.assert_swap(
        snapshot_before,
        snapshot_after,
        alice,
        alice,
        Direction::B2A,
        amount,
        receive_amount_min..=amount,
        expected_receive_amount,
        fee,
    );
}

#[test]
fn swap_more_yaro() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.001)
            .with_admin_init_deposit(500_000.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref admin,
        ..
    } = testing_env;

    pool.deposit_with_address(admin, (0.0, 250_000.0), 249_000.0);

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 10_000.0;
    // TODO: max, range, you know the drill
    let receive_amount_min = 10090.0;
    let (expected_receive_amount, fee) = testing_env.pool.receive_amount(amount, Direction::A2B);
    let expected_receive_amount = uint_to_float(expected_receive_amount, 7);

    pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B);

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Swap 10 000 yusd => yaro");

    testing_env.assert_swap(
        snapshot_before,
        snapshot_after,
        alice,
        alice,
        Direction::A2B,
        amount,
        receive_amount_min..=10_100.0,
        expected_receive_amount,
        fee,
    );
}

#[test]
fn swap_more_yusd() {
    // TODO: Same as swap_more_yaro()
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.001)
            .with_admin_init_deposit(500_000.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref admin,
        ..
    } = testing_env;

    pool.deposit_with_address(admin, (250_000.0, 0.0), 249_000.0);

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 10_000.0;
    let receive_amount_min = 995.0;
    let (expected_receive_amount, fee) = testing_env.pool.receive_amount(amount, Direction::A2B);
    let expected_receive_amount = uint_to_float(expected_receive_amount, 7);

    pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B);

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Swap 10_000 yusd => yaro");

    testing_env.assert_swap(
        snapshot_before,
        snapshot_after,
        alice,
        alice,
        Direction::A2B,
        amount,
        receive_amount_min..=amount,
        expected_receive_amount,
        fee,
    );
}

#[test]
#[should_panic = "DexContract(InsufficientReceivedAmount)"]
fn swap_insufficient_received_amount() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_fee_share(0.001));
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let amount = 1000.0;
    let receive_amount_min = 1000.0;

    pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B);
}

#[test]
fn swap_more_than_pool_balance() {
    let testing_env =
        TestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(500_000.0));
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);
    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);

    let deposit = (500_000.0, 500_000.0);
    pool.deposit(alice, deposit, 0.0);

    let amount = 1_000_000.0;

    pool.swap(alice, alice, amount, 0.0, Direction::A2B);

    pool.withdraw(alice, pool.user_lp_amount_f64(alice));
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw all");

    // TODO: Bring pool back to balance by Alice and then check if she got less money than before

    let alice_balance_after = snapshot_before.get_user_balances_sum(alice);

    assert!(alice_balance_after <= alice_balance_before);
}

#[test]
fn swap_more_than_pool_balance_b2a() {
    // TODO: Same as swap_more_than_pool_balance()
    let testing_env =
        TestingEnv::create(TestingEnvConfig::default().with_admin_init_deposit(100_000.0));
    let TestingEnv {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    let snapshot_before = Snapshot::take(&testing_env);
    let alice_balance_before = snapshot_before.get_user_balances_sum(alice);

    let deposit = (500_000.0, 500_000.0);
    pool.deposit(alice, deposit, 0.0);

    let amount = 1_000_000.0;

    pool.swap(alice, alice, amount, 0.0, Direction::B2A);

    pool.withdraw(alice, pool.user_lp_amount_f64(alice));
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Withdraw all");

    let alice_balance_after = snapshot_before.get_user_balances_sum(alice);

    assert!(alice_balance_after <= alice_balance_before);
}
