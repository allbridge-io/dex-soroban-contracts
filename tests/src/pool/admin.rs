use soroban_sdk::Env;

use crate::utils::{
    expect_auth_error, expect_contract_error, Pool, TestingEnvConfig, TestingEnvironment,
};

#[test]
fn set_admin_fee_share() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(&env, TestingEnvConfig::default());
    let TestingEnvironment { ref pool, .. } = testing_env;

    let admin_fee_share = 0.01;
    let expected_fee_share = Pool::convert_to_bp(admin_fee_share);

    pool.set_admin_fee_share(admin_fee_share).unwrap();

    assert_eq!(pool.admin_fee_share_bp(), expected_fee_share);
}

#[test]
fn set_admin_fee_share_invalid() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(&env, TestingEnvConfig::default());

    let call_result = testing_env.pool.set_admin_fee_share(100.0);

    expect_contract_error(&env, call_result, shared::Error::InvalidArg);
}

#[test]
fn set_admin_fee_share_no_auth() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(&env, TestingEnvConfig::default());

    env.mock_auths(&[]);
    expect_auth_error(&env, testing_env.pool.set_admin_fee_share(0.01));
}

#[test]
fn set_fee_share() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(&env, TestingEnvConfig::default());
    let TestingEnvironment { ref pool, .. } = testing_env;

    let fee_share = 0.01;
    let expected_fee_share = Pool::convert_to_bp(fee_share);

    pool.set_fee_share(fee_share).unwrap();

    assert_eq!(pool.fee_share_bp(), expected_fee_share);
}

#[test]
fn set_fee_share_invalid() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(&env, TestingEnvConfig::default());

    let call_result = testing_env.pool.set_fee_share(100.0);

    expect_contract_error(&env, call_result, shared::Error::InvalidArg);
}

#[test]
fn set_fee_share_no_auth() {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(&env, TestingEnvConfig::default());

    env.mock_auths(&[]);
    expect_auth_error(&env, testing_env.pool.set_fee_share(0.01));
}
