use crate::utils::{Pool, TestingEnv};

#[test]
fn set_admin_fee_share() {
    let testing_env = TestingEnv::default();
    let admin_fee_share = 0.01;
    let expected_fee_share = Pool::convert_to_bp(admin_fee_share);

    testing_env.pool.set_admin_fee_share(admin_fee_share);
    assert_eq!(testing_env.pool.admin_fee_share_bp(), expected_fee_share);
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn set_admin_fee_share_invalid() {
    TestingEnv::default().pool.set_admin_fee_share(100.0);
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn set_admin_fee_share_no_auth() {
    let testing_env = TestingEnv::default();
    testing_env.clear_mock_auth();
    testing_env.pool.set_admin_fee_share(0.01);
}

#[test]
fn set_fee_share() {
    let testing_env = TestingEnv::default();
    let fee_share = 0.01;
    let expected_fee_share = Pool::convert_to_bp(fee_share);

    testing_env.pool.set_fee_share(fee_share);
    assert_eq!(testing_env.pool.fee_share_bp(), expected_fee_share);
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn set_fee_share_invalid() {
    TestingEnv::default().pool.set_fee_share(100.0);
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn set_fee_share_no_auth() {
    let testing_env = TestingEnv::default();
    testing_env.clear_mock_auth();
    testing_env.pool.set_fee_share(0.01);
}
