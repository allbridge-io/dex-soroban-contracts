use std::{
    any::type_name,
    cmp::Ordering,
    fmt::{Debug, Display},
};

use color_print::cformat;
use soroban_sdk::{
    testutils::Events,
    xdr::{ScError, ScVal},
    Address, BytesN, ConversionError, Env, Error as SorobanError, FromVal, InvokeError, Symbol,
    TryFromVal, Val,
};

use soroban_sdk::xdr::ScAddress;

use crate::contracts::pool::Direction;

pub const SYSTEM_PRECISION: u32 = 3;

pub fn error_code_to_error(v: u32) -> shared::Error {
    // don't try this at home
    unsafe { std::mem::transmute(v) }
}

pub type CallResult<T = ()> = Result<T, SorobanError>;
pub type SorobanCallResult<T, E = ConversionError> =
    Result<Result<T, E>, Result<SorobanError, InvokeError>>;

pub fn unwrap_call_result<T>(env: &Env, call_result: CallResult<T>) -> T {
    let Err(error) = call_result else {
        return call_result.unwrap();
    };

    let val = ScVal::from_val(env, error.as_val());
    let sc_error = ScError::try_from(val).expect("Expect ScError");

    match sc_error {
        ScError::Contract(contract_error) => {
            panic!("DexContract({:?})", error_code_to_error(contract_error))
        }
        _ => panic!("{:?}", sc_error),
    }
}

pub fn desoroban_result<T, E: Debug>(soroban_result: SorobanCallResult<T, E>) -> CallResult<T> {
    soroban_result.map(Result::unwrap).map_err(Result::unwrap)
}

pub fn int_to_float(amount: i128, decimals: i32) -> f64 {
    (amount as f64) / 10.0f64.powi(decimals)
}

pub fn float_to_uint(amount: f64, decimals: u32) -> u128 {
    assert!(amount >= 0.0);
    (amount * 10.0f64.powi(decimals as i32)) as u128
}

pub fn uint_to_float(amount: u128, decimals: u32) -> f64 {
    (amount as f64) / 10.0f64.powi(decimals as i32)
}

pub fn float_to_uint_sp(amount: f64) -> u128 {
    float_to_uint(amount, SYSTEM_PRECISION)
}

pub fn uint_to_float_sp(amount: u128) -> f64 {
    uint_to_float(amount, SYSTEM_PRECISION)
}

pub fn format_diff<T: PartialOrd + Display>(start: T, to: T) -> String {
    match to.partial_cmp(&start).unwrap() {
        Ordering::Equal => cformat!("<dim>{start} => {to}</dim>"),
        Ordering::Greater => cformat!("<bright-green>{start} => {to}</bright-green>"),
        Ordering::Less => cformat!("<bright-red>{start} => {to}</bright-red>"),
    }
}

fn type_name_of_event<T: FromVal<Env, Val> + ?Sized>() -> String {
    static SPLITTERS: &[char] = &['(', ')', '[', ']', '<', '>', '{', '}', ' ', ',', '='];
    type_name::<T>()
        .split_inclusive(SPLITTERS)
        .flat_map(|component| component.rsplit("::").next())
        .collect()
}

pub fn get_latest_event<T: FromVal<Env, Val>>(env: &Env) -> Option<T> {
    env.events()
        .all()
        .iter()
        .rev()
        .find_map(|(_, topic, event_data)| {
            Symbol::try_from_val(env, &topic.last().unwrap())
                .map(|symbol| {
                    symbol
                        .to_string()
                        .eq(&type_name_of_event::<T>())
                        .then(|| T::from_val(env, &event_data))
                })
                .ok()
                .flatten()
        })
}

pub fn assert_rel_eq(a: u128, b: u128, d: u128) {
    assert!(
        a.abs_diff(b) <= d,
        "a: {}, b: {}, d: {}, diff: {}",
        a,
        b,
        d,
        a.abs_diff(b)
    );
}

pub fn contract_id(address: &Address) -> BytesN<32> {
    let sc_address: ScAddress = address.try_into().unwrap();
    if let ScAddress::Contract(c) = sc_address {
        BytesN::from_array(address.env(), &c.0)
    } else {
        panic!("address is not a contract {:?}", address);
    }
}

pub fn get_percentage(v: f64, percentage: f64) -> f64 {
    assert!((0.0..=100.0).contains(&percentage));

    v * (percentage / 100.0)
}

pub fn percentage_to_bp(percentage: f64) -> u128 {
    assert!((0.0..=100.0).contains(&percentage));

    (percentage * 100.0) as u128
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Direction::A2B => Direction::B2A,
            Direction::B2A => Direction::A2B,
        }
    }
}
