use soroban_sdk::Env;

use crate::{
    contracts::pool::Direction,
    utils::{expect_auth_error, Snapshot, TestingEnvConfig, TestingEnvironment},
};

#[test]
fn claim_admin_fee() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default()
            .with_pool_fee_share(0.01)
            .with_pool_admin_fee(0.01),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    let expected_admin_fees = (0.0099997, 0.0100002);

    pool.swap(alice, bob, 100.0, 98.0, Direction::B2A).unwrap();
    pool.swap(alice, bob, 100.0, 98.0, Direction::A2B).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_admin_fee().unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Admin claim fee"));

    TestingEnvironment::assert_claim_admin_fee(snapshot_before, snapshot_after, expected_admin_fees)
}

#[test]
fn claim_admin_fee_no_auth() {
    let env = Env::default();
    let testing_env =
        TestingEnvironment::create(&env, TestingEnvConfig::default().with_pool_admin_fee(0.01));
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
            .with_pool_fee_share(0.01)
            .with_yaro_admin_deposit(0.0)
            .with_yusd_admin_deposit(0.0),
    );
    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;
    let expected_rewards = (1.0012199, 0.9987799);

    pool.deposit(alice, (2000.0, 2000.0), 0.0).unwrap();
    pool.swap(alice, bob, 100.0, 98.0, Direction::A2B).unwrap();
    pool.swap(bob, alice, 100.0, 98.0, Direction::B2A).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_rewards(alice).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, Some("Alice claim rewards"));

    pool.invariant_total_lp_less_or_equal_d();
    // TestingEnvironment::assert_claimed_reward_event(&env, alice, expected_rewards);
    TestingEnvironment::assert_claim(snapshot_before, snapshot_after, alice, expected_rewards);
}
