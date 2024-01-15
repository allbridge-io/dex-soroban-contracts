use soroban_sdk::Env;

use crate::{
    contracts::pool::{Direction, RewardsClaimed},
    utils::{
        assert_rel_eq, float_to_int, get_latest_event, int_to_float, Snapshot, TestingEnvConfig,
        TestingEnvironment,
    },
};

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
    pool.deposit(&alice, (100.0, 50.0), 0.0).unwrap();
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
fn withdraw() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref pool,
        ref alice,
        ..
    } = testing_env;

    pool.deposit(&alice, (100.0, 50.0), 0.0).unwrap();

    let alice_lp_amount = pool.user_lp_amount(alice);
    let alice_lp_amount_float = int_to_float(alice_lp_amount);
    let (token_a_amount, token_b_amount) = pool.withdraw_amounts(alice);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.withdraw(&alice, alice_lp_amount_float).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, Some("Withdraw"));

    let alice_yaro_diff =
        int_to_float(snapshot_after.alice_yaro_balance - snapshot_before.alice_yaro_balance);
    let alice_yusd_diff =
        int_to_float(snapshot_after.alice_yusd_balance - snapshot_before.alice_yusd_balance);

    let pool_yaro_diff =
        int_to_float(snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance);
    let pool_yusd_diff =
        int_to_float(snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance);

    let total_lp_amount_diff = snapshot_before.total_lp_amount - snapshot_after.total_lp_amount;

    assert_rel_eq(total_lp_amount_diff, float_to_int(150.0), float_to_int(0.1));
    assert_eq!(alice_yusd_diff, token_a_amount);
    assert_eq!(alice_yaro_diff, token_b_amount);
    assert_eq!(pool_yusd_diff, token_a_amount);
    assert_eq!(pool_yaro_diff, token_b_amount);
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

    pool.swap(&alice, &alice, amount, receive_amount_min, Direction::A2B);

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
fn reward() {
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

    pool.deposit(&alice, (1000.0, 500.0), 0.0).unwrap();
    pool.deposit(&bob, (800.0, 750.0), 0.0).unwrap();

    println!("rewards: {:?}", get_latest_event::<RewardsClaimed>(&env));

    let amount = 1000.0;
    let receive_amount_min = 995.5;

    let snapshot_before = Snapshot::take(&testing_env);

    pool.swap(alice, bob, amount, receive_amount_min, Direction::B2A);
    pool.swap(bob, alice, amount, receive_amount_min, Direction::A2B);
    pool.swap(alice, bob, amount, receive_amount_min, Direction::B2A);

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Swap 1000 yusd => yaro"));

    println!("rewards: {:?}", get_latest_event::<RewardsClaimed>(&env));

    println!(
        "pending_reward: {:?}",
        pool.client.pending_reward(&alice.as_address())
    );

    let snapshot_before = Snapshot::take(&testing_env);

    pool.claim_rewards(alice).unwrap();

    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Alice claim rewards"));
}
