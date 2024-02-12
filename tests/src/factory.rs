use soroban_sdk::Env;

use crate::utils::{expect_auth_error, expect_contract_error, TestingEnvironment};

#[test]
fn add_new_pair() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let (yellow_token, duck_token) =
        TestingEnvironment::generate_token_pair(&env, &testing_env.admin);

    let deployed_pool = testing_env
        .factory
        .create_pair(
            &testing_env.admin,
            10,
            &yellow_token.id,
            &duck_token.id,
            10,
            10,
        )
        .unwrap();

    let pool = testing_env
        .factory
        .get_pool(&yellow_token.id, &duck_token.id)
        .unwrap();

    assert_eq!(deployed_pool, pool);
}

#[test]
fn add_new_pair_no_auth() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let (yellow_token, duck_token) =
        TestingEnvironment::generate_token_pair(&env, &testing_env.admin);

    env.mock_auths(&[]);
    let call_resulktt = testing_env.factory.create_pair(
        &testing_env.admin,
        10,
        &yellow_token.id,
        &duck_token.id,
        10,
        10,
    );

    expect_auth_error(&env, call_resulktt);
}

#[test]
fn identical_addresses() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref yaro_token,
        ref admin,
        ..
    } = testing_env;

    let call_result =
        testing_env
            .factory
            .create_pair(admin, 10, &yaro_token.id, &yaro_token.id, 10, 10);

    expect_contract_error(&env, call_result, shared::Error::IdenticalAddresses);
}

#[test]
fn pair_exist() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref yaro_token,
        ref yusd_token,
        ref admin,
        ..
    } = testing_env;

    let call_result =
        testing_env
            .factory
            .create_pair(admin, 10, &yaro_token.id, &yusd_token.id, 10, 10);

    expect_contract_error(&env, call_result, shared::Error::PairExist);

    let call_result =
        testing_env
            .factory
            .create_pair(admin, 10, &yusd_token.id, &yaro_token.id, 10, 10);

    expect_contract_error(&env, call_result, shared::Error::PairExist);
}

#[test]
fn get_pool() {
    let env = Env::default();
    let testing_env = TestingEnvironment::default(&env);
    let TestingEnvironment {
        ref yaro_token,
        ref yusd_token,
        ..
    } = testing_env;

    let pool = testing_env
        .factory
        .get_pool(&yaro_token.id, &yusd_token.id)
        .unwrap();
    assert_eq!(pool, testing_env.pool.id);

    let pool = testing_env
        .factory
        .get_pool(&yusd_token.id, &yaro_token.id)
        .unwrap();
    assert_eq!(pool, testing_env.pool.id);
}
