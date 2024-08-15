#![cfg(test)]

use soroban_sdk::{testutils::Address as _, testutils::BytesN as _, Address, BytesN};

use crate::three_pool_utils::TestingEnv;

#[test]
#[should_panic = "Context(InvalidAction)"]
fn add_new_pool_no_auth() {
    let testing_env = TestingEnv::default();
    let (token_a, token_b, token_c) =
        TestingEnv::generate_tokens(&testing_env.env, testing_env.admin.as_ref());

    testing_env.clear_mock_auth().factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &token_a.id,
        &token_b.id,
        &token_c.id,
        10,
        10,
    );
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn set_admin_no_auth() {
    let testing_env = TestingEnv::default();

    testing_env
        .clear_mock_auth()
        .factory
        .set_admin(Address::generate(&testing_env.env));
}

#[test]
fn set_admin() {
    let testing_env = TestingEnv::default();
    let new_admin = Address::generate(&testing_env.env);

    testing_env.factory.set_admin(new_admin.clone());
    assert_eq!(testing_env.factory.client.get_admin(), new_admin);
}

#[test]
fn update_two_pool_wasm_hash() {
    let testing_env = TestingEnv::default();

    let new_wasm_hash = BytesN::<32>::random(&testing_env.env);

    testing_env.factory.update_wasm_hash(&new_wasm_hash);

    assert_eq!(
        testing_env.factory.client.get_three_pool_wasm_hash(),
        new_wasm_hash
    );
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn update_wasm_hash_no_auth() {
    let testing_env = TestingEnv::default();

    testing_env
        .clear_mock_auth()
        .factory
        .update_wasm_hash(&BytesN::<32>::random(&testing_env.env));
}

#[test]
#[should_panic = "DexContract(IdenticalAddresses)"]
fn identical_addresses() {
    let testing_env = TestingEnv::default();
    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &testing_env.token_b.id,
        &testing_env.token_b.id,
        &testing_env.token_c.id,
        10,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_fee_share() {
    let testing_env = TestingEnv::default();
    let (token_a, token_b, token_c) =
        TestingEnv::generate_tokens(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &token_a.id,
        &token_b.id,
        &token_c.id,
        10_000,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_a() {
    let testing_env = TestingEnv::default();
    let (token_a, token_b, token_c) =
        TestingEnv::generate_tokens(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        65,
        &token_a.id,
        &token_b.id,
        &token_c.id,
        100,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_admin_fee_share() {
    let testing_env = TestingEnv::default();
    let (token_a, token_b, token_c) =
        TestingEnv::generate_tokens(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &token_a.id,
        &token_b.id,
        &token_c.id,
        10,
        10_000,
    );
}

#[test]
#[should_panic = "DexContract(PoolExist)"]
fn pool_exist() {
    let testing_env = TestingEnv::default();

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &testing_env.token_b.id,
        &testing_env.token_a.id,
        &testing_env.token_c.id,
        10,
        10,
    );
}

#[test]
#[should_panic = "DexContract(PoolExist)"]
fn pair_exist_reverse() {
    let testing_env = TestingEnv::default();
    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &testing_env.token_c.id,
        &testing_env.token_b.id,
        &testing_env.token_a.id,
        10,
        10,
    );
}

#[test]
fn add_new_pair() {
    let testing_env = TestingEnv::default();
    let (token_a, token_b, token_c) =
        TestingEnv::generate_tokens(&testing_env.env, testing_env.admin.as_ref());

    let deployed_pool = testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &token_a.id,
        &token_b.id,
        &token_c.id,
        10,
        10,
    );

    let pool = testing_env
        .factory
        .pool(&token_a.id, &token_b.id, &token_c.id);

    assert_eq!(deployed_pool, pool);
}

#[test]
fn get_pool() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref token_b,
        ref token_a,
        ref token_c,
        ..
    } = testing_env;

    let pool = testing_env
        .factory
        .pool(&token_b.id, &token_a.id, &token_c.id);
    assert_eq!(pool, testing_env.pool.id);

    let pool = testing_env
        .factory
        .pool(&token_a.id, &token_c.id, &token_b.id);
    assert_eq!(pool, testing_env.pool.id);
}

#[test]
#[should_panic = "DexContract(MaxPoolsNumReached)"]
fn add_new_pools() {
    let testing_env = TestingEnv::default();

    for _ in 0..20 {
        let (token_a, token_b, token_c) =
            TestingEnv::generate_tokens(&testing_env.env, testing_env.admin.as_ref());

        let _ = testing_env.factory.create_pool(
            testing_env.admin.as_ref(),
            10,
            &token_a.id,
            &token_b.id,
            &token_c.id,
            10,
            10,
        );
    }

    let (token_a, token_b, token_c) =
        TestingEnv::generate_tokens(&testing_env.env, testing_env.admin.as_ref());

    let _ = testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        &token_a.id,
        &token_b.id,
        &token_c.id,
        10,
        10,
    );
}
