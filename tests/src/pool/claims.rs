use crate::{
    contracts::pool::Direction,
    utils::{TestingEnv, TestingEnvConfig, DOUBLE_ZERO},
};

#[test]
#[should_panic(expected = "Context(InvalidAction)")]
fn claim_admin_fee_no_auth() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_admin_fee(1.0));
    let TestingEnv {
        ref pool, ref bob, ..
    } = testing_env;

    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A);
    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B);

    testing_env.clear_mock_auth().pool.claim_admin_fee();
}

#[test]
fn claim_admin_fee() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_pool_admin_fee(1.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    // Expected is 1% of 1% of 100 USD, which is around 1 cent
    let expected_admin_fees = (0.009_999_7, 0.010_000_2);

    pool.swap(alice, bob, 100.0, 98.0, Direction::B2A);
    pool.swap(alice, bob, 100.0, 99.0, Direction::A2B);

    testing_env.do_claim_admin_fee(expected_admin_fees);
    testing_env.do_claim_admin_fee(DOUBLE_ZERO);
}

#[test]
fn claim_rewards() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_admin_init_deposit(0.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;
    pool.deposit(alice, (2_000.0, 2_000.0), 0.0);

    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B);
    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A);

    // Expected 1% of 100 USD, which is around 1%
    testing_env.do_claim(alice, (1.001_219_9, 0.998_779_9));
    testing_env.do_claim(alice, DOUBLE_ZERO);
}

#[test]
fn user_and_admin_claim_rewards() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_pool_admin_fee(20.0)
            .with_admin_init_deposit(0.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    // Expected 1% of 100 USD, which is around 1%
    let _expected_total_rewards = (1.001_219_9, 0.998_779_9);
    let expected_admin_fees = (0.200_243_98, 0.199_755_98);
    let expected_user_rewards = (0.800_975_92, 0.799_023_92);

    pool.deposit(alice, (2_000.0, 2_000.0), 0.0);
    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B);
    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A);

    testing_env.do_claim(alice, expected_user_rewards);
    testing_env.do_claim_admin_fee(expected_admin_fees);
}

#[test]
fn get_rewards_after_second_claim() {
    let testing_env = TestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_admin_init_deposit(0.0),
    );
    let TestingEnv {
        ref pool,
        ref alice,
        ref bob,
        ..
    } = testing_env;

    // Expected 1% of 100 USD, which is around 1%
    let yaro_expected_reward = 0.998_779_9;
    let yusd_expected_reward = 1.001_219_9;

    pool.deposit(alice, (2_000.0, 2_000.0), 0.0);
    pool.swap(bob, bob, 100.0, 98.0, Direction::A2B);
    testing_env.do_claim(alice, (0.0, yaro_expected_reward));
    pool.swap(bob, bob, 100.0, 98.0, Direction::B2A);
    testing_env.do_claim(alice, (yusd_expected_reward, 0.0));
}
