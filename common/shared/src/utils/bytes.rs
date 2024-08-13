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

#[cfg(test)]
mod tests {
    use soroban_sdk::{Address, Env, String};
    use soroban_sdk::testutils::arbitrary::std::println;
    use crate::utils::bytes::address_to_bytes;

    #[test]
    fn test() {
        let env = Env::default();
        let address = Address::from_string(&String::from_str(&env, "GACWN434MDHQPLIUW6SPRDWTQ7BER5BTQWJGL2GDQ54IZYJHJQHODRTZ"));
        let result = address_to_bytes(&env, &address).unwrap();
        println!("{:?}", result);
    }
}