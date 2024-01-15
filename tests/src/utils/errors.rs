use soroban_sdk::xdr::{ScError, ScErrorCode, ScVal};
use soroban_sdk::{Env, FromVal};

use super::CallResult;

pub fn expect_auth_error<T>(env: &Env, call_result: CallResult<T>) {
    let error = call_result.err().expect("Expect error");
    let val = ScVal::from_val(env, error.as_val());

    match ScError::try_from(val).expect("Expect ScError") {
        ScError::Context(x) => {
            if x != ScErrorCode::InvalidAction {
                panic!("Expect ScErrorCode::InvalidAction");
            }
        }
        _ => panic!("Expect ScErrorCode::InvalidAction"),
    };
}

pub fn expect_sc_error<T>(env: &Env, call_result: CallResult<T>, sc_error: ScError) {
    let error = call_result.err().expect("Expect error");

    match ScVal::from_val(env, error.as_val()) {
        ScVal::Error(actual_error) => {
            assert_eq!(actual_error, sc_error)
        }
        _ => panic!("No ScError"),
    };
}

pub fn expect_contract_error<T>(
    env: &Env,
    call_result: CallResult<T>,
    contract_error: shared::Error,
) {
    expect_sc_error(env, call_result, ScError::Contract(contract_error as u32));
}
