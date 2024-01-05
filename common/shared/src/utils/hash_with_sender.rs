use crate::Error;
use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

use crate::utils::{bytes_to_slice, merge_slices_by_half};

pub fn hash_with_sender(env: &Env, hash: &BytesN<32>, sender: &BytesN<32>) -> BytesN<32> {
    let crypto = env.crypto();

    let message_slice = merge_slices_by_half::<32, 64>(&hash.to_array(), &sender.to_array());
    let message_bytes = Bytes::from_slice(env, &message_slice);
    let mut new_hash = crypto.keccak256(&message_bytes);

    new_hash.set(0, hash.get_unchecked(0));
    new_hash.set(1, hash.get_unchecked(1));
    new_hash
}

pub fn hash_with_sender_address(
    env: &Env,
    hash: &BytesN<32>,
    sender: &Address,
) -> Result<BytesN<32>, Error> {
    let sender_bytes = address_to_bytes(env, sender)?;
    Ok(hash_with_sender(env, hash, &sender_bytes))
}

pub fn address_to_bytes(env: &Env, sender: &Address) -> Result<BytesN<32>, Error> {
    let sender_xdr = sender.to_xdr(env);
    if sender_xdr.len() == 40 {
        let xdr_slice = bytes_to_slice::<40>(sender.to_xdr(env));
        Ok(BytesN::from_array(
            env,
            arrayref::array_ref![xdr_slice, 8, 32],
        ))
    } else if sender_xdr.len() == 44 {
        let xdr_slice = bytes_to_slice::<44>(sender.to_xdr(env));
        Ok(BytesN::from_array(
            env,
            arrayref::array_ref![xdr_slice, 12, 32],
        ))
    } else {
        Err(Error::InvalidArg)
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use std::println;

    use crate::utils::hash_with_sender_address;
    use soroban_sdk::xdr::FromXdr;
    use soroban_sdk::{Address, Bytes, BytesN, Env};

    use super::hash_with_sender;

    pub fn convert_eth_address_to_byte32(env: &Env, address: &str) -> BytesN<32> {
        extern crate alloc;
        use alloc::vec;

        let address = hex::decode(address).unwrap();
        let address = (vec![vec![0u8; 12], address]).concat();
        let address = BytesN::<32>::from_array(env, arrayref::array_ref![address, 0, 32]);

        address
    }

    #[test]
    fn get_hash_with_sender_adress_test() {
        let env = Env::default();
        let hash: BytesN<32> = BytesN::from_array(&env, &[1; 32]);
        let address = Address::from_xdr(
            &env,
            &Bytes::from_array(
                &env,
                &[
                    0, 0, 0, 18, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                ],
            ),
        )
        .unwrap();
        let result = hash_with_sender_address(&env, &hash, &address).unwrap();
        let expected_hash = "01018c96a2454213fcc0daff3c96ad0398148181b9fa6488f7ae2c0af5b20aa0";
        let mut hash_slice = [0; 32];
        hex::decode_to_slice(expected_hash, &mut hash_slice).unwrap();
        let expected_bytes = BytesN::from_array(&env, &hash_slice);
        assert_eq!(result, expected_bytes);
    }

    #[test]
    fn get_hash_with_sender_test() {
        let env = Env::default();

        let expected_hash = "070333c49eb1bd1b9cde2709d604db84f8d99e90f804daa8ae637fd8f71ae5b4";

        let message_hash =
            hex::decode("0703370f559628f02b43f1b9a9974d23ea0d5b277b0c9d60a23d33eaf0ba01e7")
                .unwrap();
        let message_hash: BytesN<32> =
            BytesN::from_array(&env, arrayref::array_ref![message_hash, 0, 32]);

        let bridge_adddress = "B3fcA30B51AE8e5488598D54240bD692025a03F4";
        let bridge_address = convert_eth_address_to_byte32(&env, bridge_adddress);

        let msg = hash_with_sender(&env, &message_hash, &bridge_address);

        let actual_hash = hex::encode(msg.to_array());

        println!("result: \t0x{}", actual_hash);

        assert_eq!(&actual_hash, expected_hash);
    }
}
