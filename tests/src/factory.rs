use crate::utils::TestingEnv;

#[test]
#[should_panic = "Context(InvalidAction)"]
fn add_new_pair_no_auth() {
    let testing_env = TestingEnv::default();
    let (yellow_token, duck_token) =
        TestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    testing_env.clear_mock_auth().factory.create_pair(
        testing_env.admin.as_ref(),
        10,
        &yellow_token.id,
        &duck_token.id,
        10,
        10,
    );
}

#[test]
#[should_panic = "DexContract(IdenticalAddresses)"]
fn identical_addresses() {
    let testing_env = TestingEnv::default();
    testing_env.factory.create_pair(
        testing_env.admin.as_ref(),
        10,
        &testing_env.yaro_token.id,
        &testing_env.yaro_token.id,
        10,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_fee_share() {
    let testing_env = TestingEnv::default();
    let (yellow, duck) =
        TestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pair(
        testing_env.admin.as_ref(),
        10,
        &yellow.id,
        &duck.id,
        10_000,
        10,
    );
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn invalid_admin_fee_share() {
    let testing_env = TestingEnv::default();
    let (yellow_token, duck_token) =
        TestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    testing_env.factory.create_pair(
        testing_env.admin.as_ref(),
        10,
        &yellow_token.id,
        &duck_token.id,
        10,
        10_000,
    );
}

#[test]
#[should_panic = "DexContract(PairExist)"]
fn pair_exist() {
    let testing_env = TestingEnv::default();

    testing_env.factory.create_pair(
        testing_env.admin.as_ref(),
        10,
        &testing_env.yaro_token.id,
        &testing_env.yusd_token.id,
        10,
        10,
    );
}

#[test]
#[should_panic = "DexContract(PairExist)"]
fn pair_exist_reverse() {
    let testing_env = TestingEnv::default();
    testing_env.factory.create_pair(
        testing_env.admin.as_ref(),
        10,
        &testing_env.yusd_token.id,
        &testing_env.yaro_token.id,
        10,
        10,
    );
}

#[test]
fn add_new_pair() {
    let testing_env = TestingEnv::default();
    let (yellow_token, duck_token) =
        TestingEnv::generate_token_pair(&testing_env.env, testing_env.admin.as_ref());

    let deployed_pool = testing_env.factory.create_pair(
        testing_env.admin.as_ref(),
        10,
        &yellow_token.id,
        &duck_token.id,
        10,
        10,
    );

    let pool = testing_env.factory.pool(&yellow_token.id, &duck_token.id);

    assert_eq!(deployed_pool, pool);
}

#[test]
fn get_pool() {
    let testing_env = TestingEnv::default();
    let TestingEnv {
        ref yaro_token,
        ref yusd_token,
        ..
    } = testing_env;

    let pool = testing_env.factory.pool(&yaro_token.id, &yusd_token.id);
    assert_eq!(pool, testing_env.pool.id);

    let pool = testing_env.factory.pool(&yusd_token.id, &yaro_token.id);
    assert_eq!(pool, testing_env.pool.id);
}
