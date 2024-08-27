#![cfg(test)]

use crate::contracts_wrappers::{TestingEnv, TestingEnvConfig};
use crate::two_pool::TwoPoolTestingEnv;

#[test]
#[should_panic(expected = "Context(InvalidAction)")]
fn claim_admin_fee_no_auth() {
    let testing_env =
        TwoPoolTestingEnv::create(TestingEnvConfig::default().with_pool_admin_fee(1.0));
    let TwoPoolTestingEnv {
        ref pool,
        ref bob,
        token_a: ref yusd_token,
        token_b: ref yaro_token,
        ..
    } = testing_env;

    pool.swap(bob, bob, 100.0, 98.0, yaro_token, yusd_token);
    pool.swap(bob, bob, 100.0, 98.0, yusd_token, yaro_token);

    testing_env.clear_mock_auth().pool.claim_admin_fee();
}

#[test]
fn claim_admin_fee() {
    let testing_env = TwoPoolTestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_pool_admin_fee(1.0),
    );
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        ref bob,
        token_a: ref yusd_token,
        token_b: ref yaro_token,
        ..
    } = testing_env;

    // Expected is 1% of 1% of 100 USD, which is around 1 cent
    let expected_admin_fees = [0.009_999_7, 0.010_000_2];

    pool.swap(alice, bob, 100.0, 98.0, yaro_token, yusd_token);
    pool.swap(alice, bob, 100.0, 99.0, yusd_token, yaro_token);

    testing_env.do_claim_admin_fee(expected_admin_fees);
    testing_env.do_claim_admin_fee([0.0; 2]);
}

#[test]
fn claim_rewards() {
    let testing_env = TwoPoolTestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_admin_init_deposit(0.0),
    );
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        ref bob,
        token_a: ref yusd_token,
        token_b: ref yaro_token,
        ..
    } = testing_env;
    pool.deposit(alice, [2_000.0, 2_000.0], 0.0);

    pool.swap(bob, bob, 100.0, 98.0, yusd_token, yaro_token);
    pool.swap(bob, bob, 100.0, 98.0, yaro_token, yusd_token);

    // Expected 1% of 100 USD, which is around 1%
    testing_env.do_claim(alice, [1.001_219_9, 0.998_779_9]);
    testing_env.do_claim(alice, [0.0; 2]);
}

#[test]
fn user_and_admin_claim_rewards() {
    let testing_env = TwoPoolTestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_pool_admin_fee(20.0)
            .with_admin_init_deposit(0.0),
    );
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        ref bob,
        token_a: ref yusd_token,
        token_b: ref yaro_token,
        ..
    } = testing_env;

    // Expected 1% of 100 USD, which is around 1%
    let _expected_total_rewards = [1.001_219_9, 0.998_779_9];
    let expected_admin_fees = [0.200_243_98, 0.199_755_98];
    let expected_user_rewards = [0.800_975_92, 0.799_023_92];

    pool.deposit(alice, [2_000.0, 2_000.0], 0.0);
    pool.swap(bob, bob, 100.0, 98.0, yusd_token, yaro_token);
    pool.swap(bob, bob, 100.0, 98.0, yaro_token, yusd_token);

    testing_env.do_claim(alice, expected_user_rewards);
    testing_env.do_claim_admin_fee(expected_admin_fees);
}

#[test]
fn get_rewards_after_second_claim() {
    let testing_env = TwoPoolTestingEnv::create(
        TestingEnvConfig::default()
            .with_pool_fee_share(1.0)
            .with_admin_init_deposit(0.0),
    );
    let TwoPoolTestingEnv {
        ref pool,
        ref alice,
        ref bob,
        token_a: ref yusd_token,
        token_b: ref yaro_token,
        ..
    } = testing_env;

    // Expected 1% of 100 USD, which is around 1%
    let yaro_expected_reward = 0.998_779_9;
    let yusd_expected_reward = 1.001_219_9;

    pool.deposit(alice, [2_000.0, 2_000.0], 0.0);
    pool.swap(bob, bob, 100.0, 98.0, yusd_token, yaro_token);
    testing_env.do_claim(alice, [0.0, yaro_expected_reward]);
    pool.swap(bob, bob, 100.0, 98.0, yaro_token, yusd_token);
    testing_env.do_claim(alice, [yusd_expected_reward, 0.0]);
}
