use soroban_sdk::{xdr::ToXdr, Address, BytesN, Env};

use crate::Error;

use super::bytes_to_slice;

pub fn address_to_bytes(env: &Env, address: &Address) -> Result<BytesN<32>, Error> {
    let sender_xdr = address.to_xdr(env);
    if sender_xdr.len() == 40 {
        let xdr_slice = bytes_to_slice::<40>(address.to_xdr(env));
        Ok(BytesN::from_array(
            env,
            arrayref::array_ref![xdr_slice, 8, 32],
        ))
    } else if sender_xdr.len() == 44 {
        let xdr_slice = bytes_to_slice::<44>(address.to_xdr(env));
        Ok(BytesN::from_array(
            env,
            arrayref::array_ref![xdr_slice, 12, 32],
        ))
    } else {
        Err(Error::InvalidArg)
    }
}
