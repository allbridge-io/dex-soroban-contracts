use crate::{
    contracts::pool::Direction,
    utils::{TestingEnv, TestingEnvConfig},
};

#[test]
fn claim_admin_fee() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.01)
            .with_pool_admin_fee(0.01),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    // Expected is 1% of 1% of 100 USD, which is around 1 cent
    let expected_admin_fees = (0.0099997, 0.0100002);

    pool.swap(alice, bob, 100.0, 98.0, Direction::B2A);
    pool.swap(alice, bob, 100.0, 99.0, Direction::A2B);

    let (snapshot_before, snapshot_after) = pool.claim_admin_fee_with_snapshots(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Admin claim fee");

    TestingEnv::assert_claim_admin_fee(snapshot_before, snapshot_after, expected_admin_fees);

    let (snapshot_before, snapshot_after) = pool.claim_admin_fee_with_snapshots(&testing_env);
    snapshot_before.print_change_with(&snapshot_after, "Admin claim fee");

    TestingEnv::assert_claim_admin_fee(snapshot_before, snapshot_after, (0.0, 0.0));
}

#[test]
#[should_panic(expected = "Context(InvalidAction)")]
fn claim_admin_fee_no_auth() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_admin_fee(0.01));
    let TestingEnv {
        ref pool, ref bob, ..
    } = testing_env;

    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A);
    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B);

    testing_env.clear_mock_auth();
    pool.claim_admin_fee();
}

#[test]
fn claim_rewards() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.01)
            .with_admin_init_deposit(0.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;
    // Expected 1% of 100 USD, which is around 1%
    let expected_rewards = (1.0012199, 0.9987799);

    pool.deposit(alice, (2_000.0, 2_000.0), 0.0);

    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B);
    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A);

    let (snapshot_before, snapshot_after) = pool.claim_rewards_with_snapshots(&testing_env, alice);
    snapshot_before.print_change_with(&snapshot_after, "Alice claim rewards");

    testing_env.assert_claim(snapshot_before, snapshot_after, alice, expected_rewards);

    let (snapshot_before, snapshot_after) = pool.claim_rewards_with_snapshots(&testing_env, alice);
    snapshot_before.print_change_with(&snapshot_after, "Second claim rewards");
    snapshot_before.assert_zero_changes(&snapshot_after);
}

// TODO: Init env with admin share and check that total rewards is equal to the sum of claimed by admin and user
//   Give admin realistic percentage (20% of all fees)

#[test]
fn get_rewards_after_second_claim() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(0.01)
            .with_admin_init_deposit(0.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;
    // Expected 1% of 100 USD, which is around 1%
    let yaro_expected_reward = 0.9987799;
    let yusd_expected_reward = 1.0012199;

    pool.deposit(alice, (2_000.0, 2_000.0), 0.0);
    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B);

    let (snapshot_before, snapshot_after) = pool.claim_rewards_with_snapshots(&testing_env, alice);
    snapshot_before.print_change_with(&snapshot_after, "Alice claim rewards");

    testing_env.assert_claim(
        snapshot_before,
        snapshot_after,
        alice,
        (0.0, yaro_expected_reward),
    );

    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A);

    let (snapshot_before, snapshot_after) = pool.claim_rewards_with_snapshots(&testing_env, alice);
    snapshot_before.print_change_with(&snapshot_after, "Second claim rewards");

    testing_env.assert_claim(
        snapshot_before,
        snapshot_after,
        alice,
        (yusd_expected_reward, 0.0),
    );
}
