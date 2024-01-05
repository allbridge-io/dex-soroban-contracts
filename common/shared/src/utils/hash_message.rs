use crate::utils::{bytes_to_slice, merge_slices_by_half};
use soroban_sdk::{Bytes, BytesN, Env, U256};

const ALLBRIDGE_MESSENGER_ID: u8 = 1;

pub fn hash_message(
    env: &Env,
    amount: u128,
    recipient: &BytesN<32>,
    source_chain_id: u32,
    destination_chain_id: u32,
    receive_token: &BytesN<32>,
    nonce: &U256,
) -> BytesN<32> {
    let crypto = env.crypto();

    let amount = merge_slices_by_half::<16, 32>(&[0u8; 32 - 16], &(amount).to_be_bytes());
    let recipient = recipient.to_array();
    let chain_bytes = bytes_to_slice::<32>(U256::from_u32(env, source_chain_id).to_be_bytes());
    let receive_token_bytes = receive_token.to_array();
    let nonce_bytes = bytes_to_slice::<32>(nonce.to_be_bytes());

    let message_bytes_array: [&[u8]; 6] = [
        &amount,
        &recipient,
        &chain_bytes,
        &receive_token_bytes,
        &nonce_bytes,
        &ALLBRIDGE_MESSENGER_ID.to_be_bytes(),
    ];

    let mut message_bytes = Bytes::new(env);

    for message_byte in message_bytes_array {
        message_bytes.extend_from_slice(message_byte);
    }

    let mut message_hash = crypto.keccak256(&message_bytes);

    message_hash.set(0, source_chain_id as u8);
    message_hash.set(1, destination_chain_id as u8);

    message_hash
}

#[cfg(test)]
mod test {
    extern crate std;

    use std::println;

    use soroban_sdk::U256;
    use soroban_sdk::{BytesN, Env};

    use crate::utils::hash_message;

    fn convert_eth_address_to_byte32(env: &Env, address: &str) -> BytesN<32> {
        extern crate alloc;
        use alloc::vec;

        let address = hex::decode(address).unwrap();
        let address = (vec![vec![0u8; 12], address]).concat();
        let address = BytesN::<32>::from_array(env, arrayref::array_ref![address, 0, 32]);

        address
    }

    #[test]
    fn hash_message_test() {
        let env = Env::default();

        let recipient = "B3fcA30B51AE8e5488598D54240bD692025a03F4";
        let receive_token = "991dc6e4965fa74135b3f67c36e46d9bcf736278";
        let amount = 100000;
        let source_chain_id = 7;
        let destination_chain_id = 3;
        let nonce = U256::from_u32(&env, 9823);

        let recipient = convert_eth_address_to_byte32(&env, recipient);
        let receive_token = convert_eth_address_to_byte32(&env, receive_token);

        println!("recipient: \t0x{}", hex::encode(recipient.to_array()));
        println!(
            "receive_token: \t0x{}",
            hex::encode(receive_token.to_array())
        );

        let result = hash_message(
            &env,
            amount,
            &recipient,
            source_chain_id,
            destination_chain_id,
            &receive_token,
            &nonce,
        );

        let actual = hex::encode(result.to_array());
        let expected = "0703370f559628f02b43f1b9a9974d23ea0d5b277b0c9d60a23d33eaf0ba01e7";

        println!("result: \t0x{}", actual);

        assert_eq!(&actual, expected);
    }
}
