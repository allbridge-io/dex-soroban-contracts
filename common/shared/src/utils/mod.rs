use soroban_sdk::Bytes;

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

pub fn merge_slices_by_half<const N: usize, const R: usize>(a: &[u8; N], b: &[u8; N]) -> [u8; R] {
    let mut slice = [0u8; R];

    slice[..N].copy_from_slice(a);
    slice[N..].copy_from_slice(b);

    slice
}

#[inline]
pub fn safe_cast<T, K: TryFrom<T>>(from: T) -> Result<K, Error> {
    K::try_from(from).map_err(|_| Error::CastFailed)
}
