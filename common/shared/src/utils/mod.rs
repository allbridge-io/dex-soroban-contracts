use soroban_sdk::{Bytes, Env, IntoVal, TryFromVal, Val, Vec};

pub mod bytes;
mod extend_ttl;
pub mod num;
pub mod require;

pub use extend_ttl::*;

use crate::Error;

pub fn bytes_to_slice<const N: usize>(bytes: Bytes) -> [u8; N] {
    let mut xdr_slice: [u8; N] = [0; N];
    bytes.copy_into_slice(&mut xdr_slice);

    xdr_slice
}

#[inline]
pub fn safe_cast<T, K: TryFrom<T>>(from: T) -> Result<K, Error> {
    K::try_from(from).map_err(|_| Error::CastFailed)
}

pub fn vec_to_array<const N: usize, T>(source: Vec<T>) -> [T; N]
where
    T: Default + Clone + Copy + IntoVal<Env, Val> + TryFromVal<Env, Val>,
{
    let mut output = [T::default(); N];

    for index in 0..N {
        output[index] = source.get_unchecked(index as u32);
    }

    output
}
