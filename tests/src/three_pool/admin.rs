#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address};

use crate::{three_pool::ThreePoolTestingEnv, utils::percentage_to_bp};

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn set_admin_fee_share_invalid() {
    ThreePoolTestingEnv::default()
        .pool
        .set_admin_fee_share(100.0);
}

#[test]
fn set_admin_fee_share() {
    let testing_env = ThreePoolTestingEnv::default();
    let admin_fee_share = 1.0;
    let expected_fee_share = percentage_to_bp(admin_fee_share);

    testing_env.pool.set_admin_fee_share(admin_fee_share);
    assert_eq!(testing_env.pool.admin_fee_share_bp(), expected_fee_share);
}

#[test]
fn set_fee_share() {
    let testing_env = ThreePoolTestingEnv::default();
    let fee_share = 0.01;
    let expected_fee_share = percentage_to_bp(fee_share);

    testing_env.pool.set_fee_share(fee_share);
    assert_eq!(testing_env.pool.fee_share_bp(), expected_fee_share);
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn set_admin_no_auth() {
    let testing_env = ThreePoolTestingEnv::default();

    testing_env
        .clear_mock_auth()
        .pool
        .set_admin(Address::generate(&testing_env.env));
}

#[test]
fn set_admin() {
    let testing_env = ThreePoolTestingEnv::default();
    let new_admin = Address::generate(&testing_env.env);

    testing_env.pool.set_admin(new_admin.clone());
    assert_eq!(testing_env.pool.client.get_admin(), new_admin);
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn set_admin_fee_share_no_auth() {
    let testing_env = ThreePoolTestingEnv::default();
    testing_env.clear_mock_auth().pool.set_admin_fee_share(1.0);
}

#[test]
#[should_panic = "DexContract(InvalidArg)"]
fn set_fee_share_invalid() {
    ThreePoolTestingEnv::default().pool.set_fee_share(100.0);
}

#[test]
#[should_panic = "Context(InvalidAction)"]
fn set_fee_share_no_auth() {
    let testing_env = ThreePoolTestingEnv::default();
    testing_env.clear_mock_auth().pool.set_fee_share(1.0);
}
