use proc_macros::{extend_ttl_info_instance, Instance, SorobanData, SorobanSimpleData, SymbolKey};
use shared::{utils::bytes::address_to_bytes, Error};
use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Map, Vec};

pub const MAX_PAIRS_NUM: u32 = 21;

#[contracttype]
#[derive(SorobanData, SorobanSimpleData, SymbolKey, Instance)]
#[extend_ttl_info_instance]
pub struct FactoryInfo {
    pub two_pool_wasm_hash: soroban_sdk::BytesN<32>,
    pub three_pool_wasm_hash: soroban_sdk::BytesN<32>,
    pub pools: Map<Vec<Address>, Address>,
}

impl FactoryInfo {
    pub fn new(
        env: &Env,
        two_pool_wasm_hash: BytesN<32>,
        three_pool_wasm_hash: BytesN<32>,
    ) -> Self {
        FactoryInfo {
            two_pool_wasm_hash,
            three_pool_wasm_hash,
            pools: Map::new(env),
        }
    }

    pub fn sort_tokens(mut v: Vec<Address>) -> Vec<Address> {
        for i in 0..v.len() {
            for j in 0..v.len() - 1 - i {
                let a = v.get_unchecked(j);
                let b = v.get_unchecked(j + 1);
                if a > b {
                    v.set(j, b);
                    v.set(j + 1, a);
                }
            }
        }

        v
    }

    pub fn merge_addresses(tokens: Vec<Address>) -> Result<Bytes, Error> {
        let env = tokens.env();
        let mut result = Bytes::new(env);

        for token in tokens.iter() {
            let address_bytes = address_to_bytes(env, &token)?;
            result.extend_from_array(&address_bytes.to_array());
        }

        Ok(result)
    }

    pub fn add_pool(&mut self, tokens: Vec<Address>, pool: &Address) {
        self.pools.set(tokens, pool.clone());
    }

    pub fn get_pools(&self) -> Result<Map<Address, Vec<Address>>, Error> {
        let mut map = Map::new(self.pools.env());

        self.pools.iter().for_each(|(tokens, pool)| {
            map.set(pool, tokens);
        });

        Ok(map)
    }

    pub fn get_pool(&self, mut tokens: Vec<Address>) -> Result<Address, Error> {
        tokens = FactoryInfo::sort_tokens(tokens);

        self.pools.get(tokens).ok_or(Error::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::factory_info::FactoryInfo;
    use soroban_sdk::{vec, Address, Bytes, Env, String};

    #[test]
    fn test_merge_addresses() {
        let env = Env::default();

        let address_a = Address::from_string(&String::from_str(
            &env,
            "GAE73XQO7ONPTIJAF2S5RBCWSG2G7HWSREOP4UDXLHWBZEDBUIIQZ3Y7",
        ));
        let address_b = Address::from_string(&String::from_str(
            &env,
            "GCBJR4SJIVIRMVAOWFMSGAOCLDU6TVEIITJOO4NVAZ6RI3FC32E5RWP2",
        ));
        let address_c = Address::from_string(&String::from_str(
            &env,
            "GACWN434MDHQPLIUW6SPRDWTQ7BER5BTQWJGL2GDQ54IZYJHJQHODRTZ",
        ));

        let result =
            FactoryInfo::merge_addresses(vec![&env, address_a, address_b, address_c]).unwrap();
        let expected = Bytes::from_slice(
            &env,
            &[
                9, 253, 222, 14, 251, 154, 249, 161, 32, 46, 165, 216, 132, 86, 145, 180, 111, 158,
                210, 137, 28, 254, 80, 119, 89, 236, 28, 144, 97, 162, 17, 12, 130, 152, 242, 73,
                69, 81, 22, 84, 14, 177, 89, 35, 1, 194, 88, 233, 233, 212, 136, 68, 210, 231, 113,
                181, 6, 125, 20, 108, 162, 222, 137, 216, 5, 102, 243, 124, 96, 207, 7, 173, 20,
                183, 164, 248, 142, 211, 135, 194, 72, 244, 51, 133, 146, 101, 232, 195, 135, 120,
                140, 225, 39, 76, 14, 225,
            ],
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_sort_tokens() {
        let env = Env::default();

        let address_a = Address::from_string(&String::from_str(
            &env,
            "GAE73XQO7ONPTIJAF2S5RBCWSG2G7HWSREOP4UDXLHWBZEDBUIIQZ3Y7",
        ));
        let address_b = Address::from_string(&String::from_str(
            &env,
            "GCBJR4SJIVIRMVAOWFMSGAOCLDU6TVEIITJOO4NVAZ6RI3FC32E5RWP2",
        ));
        let address_c = Address::from_string(&String::from_str(
            &env,
            "GACWN434MDHQPLIUW6SPRDWTQ7BER5BTQWJGL2GDQ54IZYJHJQHODRTZ",
        ));

        let expected = vec![
            &env,
            address_c.clone(),
            address_a.clone(),
            address_b.clone(),
        ];

        assert_eq!(
            FactoryInfo::sort_tokens(vec![
                &env,
                address_a.clone(),
                address_b.clone(),
                address_c.clone()
            ]),
            expected
        );
        assert_eq!(
            FactoryInfo::sort_tokens(vec![
                &env,
                address_a.clone(),
                address_c.clone(),
                address_b.clone()
            ]),
            expected
        );
        assert_eq!(
            FactoryInfo::sort_tokens(vec![
                &env,
                address_b.clone(),
                address_a.clone(),
                address_c.clone()
            ]),
            expected
        );
        assert_eq!(
            FactoryInfo::sort_tokens(vec![
                &env,
                address_b.clone(),
                address_c.clone(),
                address_a.clone()
            ]),
            expected
        );
        assert_eq!(
            FactoryInfo::sort_tokens(vec![
                &env,
                address_c.clone(),
                address_a.clone(),
                address_b.clone()
            ]),
            expected
        );
        assert_eq!(
            FactoryInfo::sort_tokens(vec![&env, address_c, address_b, address_a]),
            expected
        );
    }
}
