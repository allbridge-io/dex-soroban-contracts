use soroban_sdk::Env;

use crate::{
    contracts::pool::{Direction, RewardsClaimed},
    utils::{
        assert_rel_eq, assert_rel_eq_f64, expect_auth_error, expect_contract_error, float_to_int,
        get_latest_event, int_to_float, Snapshot, TestingEnvConfig, TestingEnvironment,
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

    let call_result = pool.deposit(alice, (10.0, 0.0), 100.0);
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

    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, (100.0, 50.0), 0.0).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Deposit: 100 yusd, 50 yaro"));

    let alice_yaro_diff =
        int_to_float(snapshot_before.alice_yaro_balance - snapshot_after.alice_yaro_balance);
    let alice_yusd_diff =
        int_to_float(snapshot_before.alice_yusd_balance - snapshot_after.alice_yusd_balance);

    let pool_yaro_diff =
        int_to_float(snapshot_after.pool_yaro_balance - snapshot_before.pool_yaro_balance);
    let pool_yusd_diff =
        int_to_float(snapshot_after.pool_yusd_balance - snapshot_before.pool_yusd_balance);
    let total_lp_amount_diff = snapshot_after.total_lp_amount - snapshot_before.total_lp_amount;

    assert_rel_eq(total_lp_amount_diff, float_to_int(150.0), float_to_int(0.1));
    assert_eq!(alice_yaro_diff, 50.0);
    assert_eq!(alice_yusd_diff, 100.0);
    assert_eq!(pool_yaro_diff, 50.0);
    assert_eq!(pool_yusd_diff, 100.0);

    println!(
        "rewards: {:?}",
        get_latest_event::<RewardsClaimed>(&env).map(|x| x.rewards)
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

    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, (100.0, 0.0), 0.0).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Deposit: 100 yusd, 50 yaro"));

    let alice_yaro_diff =
        int_to_float(snapshot_before.alice_yaro_balance - snapshot_after.alice_yaro_balance);
    let alice_yusd_diff =
        int_to_float(snapshot_before.alice_yusd_balance - snapshot_after.alice_yusd_balance);

    let pool_yaro_diff =
        int_to_float(snapshot_after.pool_yaro_balance - snapshot_before.pool_yaro_balance);
    let pool_yusd_diff =
        int_to_float(snapshot_after.pool_yusd_balance - snapshot_before.pool_yusd_balance);
    let total_lp_amount_diff = snapshot_after.total_lp_amount - snapshot_before.total_lp_amount;

    assert_rel_eq(total_lp_amount_diff, float_to_int(100.0), float_to_int(0.1));
    assert_eq!(alice_yaro_diff, 0.0);
    assert_eq!(alice_yusd_diff, 100.0);
    assert_eq!(pool_yaro_diff, 0.0);
    assert_eq!(pool_yusd_diff, 100.0);

    println!(
        "rewards: {:?}",
        get_latest_event::<RewardsClaimed>(&env).map(|x| x.rewards)
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

    pool.deposit(alice, (100.0, 50.0), 0.0).unwrap();

    let alice_lp_amount = pool.user_lp_amount(alice);
    let alice_lp_amount_float = int_to_float(alice_lp_amount);
    let (token_a_amount, token_b_amount) = pool.withdraw_amounts(alice);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(alice, alice_lp_amount_float).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    let alice_yaro_diff = snapshot_after.alice_yaro_balance - snapshot_before.alice_yaro_balance;
    let alice_yusd_diff =
        int_to_float(snapshot_after.alice_yusd_balance - snapshot_before.alice_yusd_balance);

    let pool_yaro_diff =
        int_to_float(snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance);
    let pool_yusd_diff =
        int_to_float(snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance);

    let total_lp_amount_diff = snapshot_before.total_lp_amount - snapshot_after.total_lp_amount;

    assert_rel_eq(total_lp_amount_diff, float_to_int(150.0), float_to_int(0.1));
    assert_eq!(alice_yusd_diff, token_a_amount);
    assert_rel_eq(
        alice_yaro_diff,
        float_to_int(token_b_amount),
        float_to_int(0.001),
    );
    assert_eq!(pool_yusd_diff, token_a_amount);
    assert_eq!(pool_yaro_diff, token_b_amount);
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

    let alice_lp_amount = pool.user_lp_amount(alice);
    let alice_lp_amount_float = int_to_float(alice_lp_amount);

    let call_result = pool.withdraw(alice, alice_lp_amount_float);

    expect_contract_error(&env, call_result, shared::Error::ZeroChanges)
}

#[test]
fn swap() {
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

    let snapshot_before = Snapshot::take(&testing_env);

    let amount = 1000.0;
    let receive_amount_min = 995.5;

    pool.swap(alice, alice, amount, receive_amount_min, Direction::A2B)
        .unwrap();

    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Swap 1000 yusd => yaro"));

    let alice_yaro_diff =
        int_to_float(snapshot_after.alice_yaro_balance - snapshot_before.alice_yaro_balance);
    let alice_yusd_diff =
        int_to_float(snapshot_before.alice_yusd_balance - snapshot_after.alice_yusd_balance);

    let pool_yaro_diff =
        int_to_float(snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance);
    let pool_yusd_diff =
        int_to_float(snapshot_after.pool_yusd_balance - snapshot_before.pool_yusd_balance);

    assert!(alice_yaro_diff > receive_amount_min && alice_yaro_diff <= amount);
    assert!(pool_yaro_diff > receive_amount_min && pool_yaro_diff <= amount);
    assert_eq!(alice_yusd_diff, amount);
    assert_eq!(pool_yusd_diff, amount);
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

    let amount = 100.0;
    let receive_amount_min = 98.0;
    let expected_yaro_admin_fee = 0.0100002;
    let expected_yusd_admin_fee = 0.0099997;

    pool.swap(alice, bob, amount, receive_amount_min, Direction::B2A)
        .unwrap();
    pool.swap(alice, bob, amount, receive_amount_min, Direction::A2B)
        .unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_admin_fee().unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Admin claim fee"));

    let admin_yaro_diff =
        int_to_float(snapshot_after.admin_yaro_balance - snapshot_before.admin_yaro_balance);
    let admin_yusd_diff =
        int_to_float(snapshot_after.admin_yusd_balance - snapshot_before.admin_yusd_balance);

    let pool_yaro_diff =
        int_to_float(snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance);
    let pool_yusd_diff =
        int_to_float(snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance);

    assert_rel_eq_f64(admin_yaro_diff, expected_yaro_admin_fee, 0.0001);
    assert_rel_eq_f64(admin_yusd_diff, expected_yusd_admin_fee, 0.0001);
    assert_rel_eq_f64(pool_yaro_diff, expected_yaro_admin_fee, 0.0001);
    assert_rel_eq_f64(pool_yusd_diff, expected_yaro_admin_fee, 0.0001);
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

    pool.deposit(alice, (2000.0, 2000.0), 0.0).unwrap();
    let amount = 100.0;
    let receive_amount_min = 98.0;
    let expected_yusd_reward = 1.0012208;
    let expected_yaro_reward = 0.9987789;

    let snapshot_before = Snapshot::take(&testing_env);

    pool.swap(alice, bob, amount, receive_amount_min, Direction::A2B)
        .unwrap();
    pool.swap(bob, alice, amount, receive_amount_min, Direction::B2A)
        .unwrap();

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, None);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_rewards(alice).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Alice claim rewards"));

    let rewards = get_latest_event::<RewardsClaimed>(&env).unwrap();

    assert_eq!(rewards.user, alice.as_address());
    assert_eq!(int_to_float(rewards.rewards.0), expected_yusd_reward);
    assert_eq!(int_to_float(rewards.rewards.1), expected_yaro_reward);

    let alice_yusd_diff = snapshot_after.alice_yusd_balance - snapshot_before.alice_yusd_balance;
    let pool_yusd_diff = snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance;

    assert_eq!(int_to_float(alice_yusd_diff), expected_yusd_reward);
    assert_eq!(int_to_float(pool_yusd_diff), expected_yusd_reward);

    let alice_yaro_diff = snapshot_after.alice_yaro_balance - snapshot_before.alice_yaro_balance;
    let pool_yaro_diff = snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance;

    assert_eq!(int_to_float(alice_yaro_diff), expected_yaro_reward);
    assert_eq!(int_to_float(pool_yaro_diff), expected_yaro_reward);
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

    pool.deposit(alice, (2000.0, 2000.0), 0.0).unwrap();
    let amount = 100.0;
    let receive_amount_min = 98.0;
    let expected_yusd_reward = 1.0012208;
    let expected_yaro_reward = 0.9987789;

    pool.swap(alice, bob, amount, receive_amount_min, Direction::A2B)
        .unwrap();
    pool.swap(bob, alice, amount, receive_amount_min, Direction::B2A)
        .unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.deposit(alice, (2000.0, 2000.0), 0.0).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, None);

    let rewards = get_latest_event::<RewardsClaimed>(&env).unwrap();

    assert_eq!(rewards.user, alice.as_address());
    assert_eq!(int_to_float(rewards.rewards.0), expected_yusd_reward);
    assert_eq!(int_to_float(rewards.rewards.1), expected_yaro_reward);

    let alice_yusd_diff = 2_000.0
        - int_to_float(snapshot_before.alice_yusd_balance - snapshot_after.alice_yusd_balance);
    let pool_yusd_diff = 2_000.0
        - int_to_float(snapshot_after.pool_yusd_balance - snapshot_before.pool_yusd_balance);

    assert_rel_eq_f64(alice_yusd_diff, expected_yusd_reward, 0.0001);
    assert_rel_eq_f64(pool_yusd_diff, expected_yusd_reward, 0.0001);

    let alice_yaro_diff = 2_000.0
        - int_to_float(snapshot_before.alice_yaro_balance - snapshot_after.alice_yaro_balance);
    let pool_yaro_diff = 2_000.0
        - int_to_float(snapshot_after.pool_yaro_balance - snapshot_before.pool_yaro_balance);

    assert_rel_eq_f64(alice_yaro_diff, expected_yaro_reward, 0.0001);
    assert_rel_eq_f64(pool_yaro_diff, expected_yaro_reward, 0.0001);
}
