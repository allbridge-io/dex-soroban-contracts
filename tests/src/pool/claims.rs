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

    // Expected is 1% of 1% of 100 USD, which is around 1 cent
    let expected_admin_fees = (0.0099997, 0.0100002);

    pool.swap(alice, bob, 100.0, 98.0, Direction::B2A).unwrap();
    pool.swap(alice, bob, 100.0, 99.0, Direction::A2B).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_admin_fee().unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Admin claim fee");

    TestingEnvironment::assert_claim_admin_fee(
        snapshot_before,
        snapshot_after,
        expected_admin_fees,
    );

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_admin_fee().unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Admin claim fee");

    TestingEnvironment::assert_claim_admin_fee(snapshot_before, snapshot_after, (0.0, 0.0));
}

#[test]
fn claim_admin_fee_no_auth() {
    let env = Env::default();
    let testing_env =
        TestingEnvironment::create(&env, TestingEnvConfig::default().with_pool_admin_fee(0.01));
    let TestingEnvironment {
        ref pool, ref bob, ..
    } = testing_env;

    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A).unwrap();
    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B).unwrap();

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
    // Expected 1% of 100 USD, which is around 1%
    let expected_rewards = (1.0012199, 0.9987799);

    pool.deposit(alice, (2000.0, 2000.0), 0.0).unwrap();

    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B).unwrap();
    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A).unwrap();

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_rewards(alice).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Alice claim rewards");

    pool.assert_total_lp_less_or_equal_d();
    TestingEnvironment::assert_claimed_reward_event(&env, alice, expected_rewards);
    TestingEnvironment::assert_claim(snapshot_before, snapshot_after, alice, expected_rewards);

    let snapshot_before = Snapshot::take(&testing_env);
    pool.claim_rewards(alice).unwrap();
    let snapshot_after = Snapshot::take(&testing_env);

    snapshot_before.print_change_with(&snapshot_after, "Second claim rewards");

    assert_eq!(
        snapshot_before.alice_yusd_balance,
        snapshot_after.alice_yusd_balance
    );
    assert_eq!(
        snapshot_before.alice_yaro_balance,
        snapshot_after.alice_yaro_balance
    );
    assert_eq!(
        snapshot_before.pool_yusd_balance,
        snapshot_after.pool_yusd_balance
    );
    assert_eq!(
        snapshot_before.pool_yaro_balance,
        snapshot_after.pool_yaro_balance
    );
    assert_eq!(snapshot_before.d, snapshot_after.d);
    assert_eq!(
        snapshot_before.total_lp_amount,
        snapshot_after.total_lp_amount
    );
}

// TODO: Init env with admin share and check that total rewards is equal to the sum of claimed by admin and user
//   Give admin realistic percentage (20% of all fees)

// TODO: Long test
// - Deposit
// - Swap
// - Claim
// - Swap
// - Claim again, you can continue (claim_rewards test), check if there are new rewards after second swap session
