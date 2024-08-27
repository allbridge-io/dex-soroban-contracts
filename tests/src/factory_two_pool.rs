#![cfg(test)]

use soroban_sdk::{testutils::Address as _, testutils::BytesN as _, Address, BytesN};

use crate::two_pool::TwoPoolTestingEnv;

#[test]
#[should_panic = "Context(InvalidAction)"]
fn add_new_pool_no_auth() {
    let testing_env = TwoPoolTestingEnv::default();
    let [yellow_token, duck_token] =
        TwoPoolTestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    testing_env.clear_mock_auth().factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [yellow_token.id, duck_token.id],
        10,
        10,
    );
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn set_admin_no_auth() {
    let testing_env = TwoPoolTestingEnv::default();

    testing_env
        .clear_mock_auth()
        .factory
        .set_admin(Address::generate(&testing_env.env));
}

#[test]
fn set_admin() {
    let testing_env = TwoPoolTestingEnv::default();
    let new_admin = Address::generate(&testing_env.env);

    testing_env.factory.set_admin(new_admin.clone());
    assert_eq!(testing_env.factory.client.get_admin(), new_admin);
}

#[test]
fn update_wasm_hash() {
    let testing_env = TwoPoolTestingEnv::default();

    let new_wasm_hash = BytesN::<32>::random(&testing_env.env);

    testing_env.factory.update_two_wasm_hash(&new_wasm_hash);

    assert_eq!(
        testing_env.factory.client.get_two_pool_wasm_hash(),
        new_wasm_hash
    );
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn update_wasm_hash_no_auth() {
    let testing_env = TwoPoolTestingEnv::default();

    testing_env
        .clear_mock_auth()
        .factory
        .update_two_wasm_hash(&BytesN::<32>::random(&testing_env.env));
}

#[test]
#[should_panic = "DexContract(IdenticalAddresses)"]
fn identical_addresses() {
    let testing_env = TwoPoolTestingEnv::default();
    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [
            testing_env.token_b.id.clone(),
            testing_env.token_b.id.clone(),
        ],
        10,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_fee_share() {
    let testing_env = TwoPoolTestingEnv::default();
    let [yellow, duck] =
        TwoPoolTestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [yellow.id, duck.id],
        10_000,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_a() {
    let testing_env = TwoPoolTestingEnv::default();
    let [yellow, duck] =
        TwoPoolTestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        65,
        [yellow.id, duck.id],
        100,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_admin_fee_share() {
    let testing_env = TwoPoolTestingEnv::default();
    let [yellow_token, duck_token] =
        TwoPoolTestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [yellow_token.id, duck_token.id],
        10,
        10_000,
    );
}

#[test]
#[should_panic = "DexContract(PoolExist)"]
fn pair_exist() {
    let testing_env = TwoPoolTestingEnv::default();

    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [testing_env.token_b.id, testing_env.token_a.id],
        10,
        10,
    );
}

#[test]
#[should_panic = "DexContract(PoolExist)"]
fn pair_exist_reverse() {
    let testing_env = TwoPoolTestingEnv::default();
    testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [testing_env.token_a.id, testing_env.token_b.id],
        10,
        10,
    );
}

#[test]
fn add_new_pair() {
    let testing_env = TwoPoolTestingEnv::default();
    let [yellow_token, duck_token] =
        TwoPoolTestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    let deployed_pool = testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [yellow_token.id.clone(), duck_token.id.clone()],
        10,
        10,
    );

    let pool = testing_env.factory.pool([yellow_token.id, duck_token.id]);

    assert_eq!(deployed_pool, pool);
}

#[test]
fn get_pool() {
    let testing_env = TwoPoolTestingEnv::default();
    let TwoPoolTestingEnv {
        token_b: ref yaro_token,
        token_a: ref yusd_token,
        ..
    } = testing_env;

    let pool = testing_env
        .factory
        .pool([yaro_token.id.clone(), yusd_token.id.clone()]);
    assert_eq!(pool, testing_env.pool.id);

    let pool = testing_env
        .factory
        .pool([yusd_token.id.clone(), yaro_token.id.clone()]);
    assert_eq!(pool, testing_env.pool.id);
}

#[test]
#[should_panic = "DexContract(MaxPoolsNumReached)"]
fn add_new_pairs() {
    let testing_env = TwoPoolTestingEnv::default();

    for _ in 0..20 {
        let [first_token, second_token] =
            TwoPoolTestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

        let _ = testing_env.factory.create_pool(
            testing_env.admin.as_ref(),
            10,
            [first_token.id, second_token.id],
            10,
            10,
        );
    }

    let [first_token, second_token] =
        TwoPoolTestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    let _ = testing_env.factory.create_pool(
        testing_env.admin.as_ref(),
        10,
        [first_token.id, second_token.id],
        10,
        10,
    );
}
