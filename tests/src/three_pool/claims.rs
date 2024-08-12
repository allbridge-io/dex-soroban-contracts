use crate::{
    three_pool_utils::{TestingEnv, TestingEnvConfig, TRIPLE_ZERO},
};

#[test]
#[should_panic(expected = "Context(InvalidAction)")]
fn claim_admin_fee_no_auth() {
    let testing_env = TestingEnv::create(TestingEnvConfig::default().with_pool_admin_fee(1.0));
    let TestingEnv {
        ref pool, ref bob, ref token_a, ref token_b, ..
    } = testing_env;

    pool.swap(bob, bob, 100.0, 98.0, token_b, token_a);
    pool.swap(bob, bob, 100.0, 98.0, token_b, token_a);

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
        ref token_a,
        ref token_b,
        ref token_c,
        ..
    } = testing_env;

    // Expected is 1% of 1% of 100 USD, which is around 1 cent
    let expected_admin_fees = (0.01, 0.01, 0.01);

    pool.swap(alice, bob, 100.0, 98.0, token_a, token_b);
    pool.swap(alice, bob, 100.0, 98.0, token_b, token_c);
    pool.swap(alice, bob, 100.0, 98.0, token_c, token_a);

    testing_env.do_claim_admin_fee(expected_admin_fees);
    testing_env.do_claim_admin_fee(TRIPLE_ZERO);
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
        ref token_a,
        ref token_b,
        ref token_c,
        ..
    } = testing_env;
    pool.deposit(alice, (2_000.0, 2_000.0, 2_000.0), 0.0);

    pool.swap(bob, bob, 100.0, 98.0, token_a, token_b);
    pool.swap(bob, bob, 100.0, 98.0, token_b, token_c);
    pool.swap(bob, bob, 100.0, 98.0, token_c, token_a);

    // Expected 1% of 100 USD, which is around 1%
    testing_env.do_claim(alice, (1.000_269_9, 0.999_729_9, 0.999_999_9));
    testing_env.do_claim(alice, TRIPLE_ZERO);
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
        ref token_a,
        ref token_b,
        ref token_c,
        ..
    } = testing_env;

    // Expected 1% of 100 USD, which is around 1%
    let expected_admin_fees = (0.200_054, 0.199_946, 0.2);
    let expected_user_rewards = (0.800_215_9, 0.799_783_9, 0.799_999_9);

    pool.deposit(alice, (2_000.0, 2_000.0, 2_000.0), 0.0);
    pool.swap(bob, bob, 100.0, 98.0, token_a, token_b);
    pool.swap(bob, bob, 100.0, 98.0, token_b, token_c);
    pool.swap(bob, bob, 100.0, 98.0, token_c, token_a);

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
        ref token_a,
        ref token_b,
        ref token_c,
        ..
    } = testing_env;

    // Expected 1% of 100 USD, which is around 1%
    let b_expected_reward = 0.999_729_9;
    let a_expected_reward = 1.000_269_9;
    let c_expected_reward = 0.999_999_9;

    pool.deposit(alice, (2_000.0, 2_000.0, 2_000.0), 0.0);
    pool.swap(bob, bob, 100.0, 98.0, token_a, token_b);
    testing_env.do_claim(alice, (0.0, b_expected_reward, 0.0));
    pool.swap(bob, bob, 100.0, 98.0, token_b, token_c);
    testing_env.do_claim(alice, (0.0, 0.0, c_expected_reward));
    pool.swap(bob, bob, 100.0, 98.0, token_c, token_a);
    testing_env.do_claim(alice, (a_expected_reward, 0.0, 0.0));
}
