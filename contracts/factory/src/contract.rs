use shared::{utils::extend_ttl_instance, Error};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, Vec};
use storage::Admin;

use crate::methods::public::{create_three_pool, create_two_pool, get_admin, get_three_pool, get_three_pool_wasm_hash, get_three_pools, get_two_pool, get_two_pool_wasm_hash, get_two_pools, initialize, set_admin, update_three_pool_wasm_hash, update_two_pool_wasm_hash};

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    pub fn initialize(env: Env, two_pool_wasm_hash: BytesN<32>, three_pool_wasm_hash: BytesN<32>, admin: Address) -> Result<(), Error> {
        initialize(env, two_pool_wasm_hash, three_pool_wasm_hash, admin)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_pool(
        env: Env,
        deployer: Address,
        pool_admin: Address,
        a: u128,
        tokens: Vec<Address>,
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Result<Address, Error> {
        extend_ttl_instance(&env);

        match tokens.len() {
            2 => create_two_pool(
                env,
                deployer,
                pool_admin,
                a,
                tokens.get_unchecked(0),
                tokens.get_unchecked(1),
                fee_share_bp,
                admin_fee_share_bp,
            ),
            3 => create_three_pool(
                env,
                deployer,
                pool_admin,
                a,
                tokens.get_unchecked(0),
                tokens.get_unchecked(1),
                tokens.get_unchecked(2),
                fee_share_bp,
                admin_fee_share_bp,
            ),
            _ => Err(Error::MaxPoolsNumReached)
        }
    }
    // ----------- Admin -----------

    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_admin(env, new_admin)
    }

    // ----------- View -----------

    pub fn pool(env: Env, tokens: Vec<Address>) -> Result<Address, Error> {
        extend_ttl_instance(&env);

        match tokens.len() {
            2 => get_two_pool(env, &tokens.get_unchecked(0), &tokens.get_unchecked(1)),
            3 => get_three_pool(env, &tokens.get_unchecked(0), &tokens.get_unchecked(1), &tokens.get_unchecked(2)),
            _ => Err(Error::MaxPoolsNumReached)
        }
    }

    pub fn two_pools(env: Env) -> Result<Map<Address, (Address, Address)>, Error> {
        extend_ttl_instance(&env);

        get_two_pools(env)
    }

    pub fn three_pools(env: Env) -> Result<Map<Address, (Address, Address, Address)>, Error> {
        extend_ttl_instance(&env);

        get_three_pools(env)
    }

    pub fn get_two_pool_wasm_hash(env: Env) -> Result<BytesN<32>, Error> {
        get_two_pool_wasm_hash(env)
    }
    pub fn get_three_pool_wasm_hash(env: Env) -> Result<BytesN<32>, Error> {
        get_three_pool_wasm_hash(env)
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        get_admin(env)
    }

    // ----------- Upgrade -----------

    pub fn update_two_pool_wasm_hash(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        extend_ttl_instance(&env);

        update_two_pool_wasm_hash(env, new_wasm_hash)
    }

    pub fn update_three_pool_wasm_hash(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        extend_ttl_instance(&env);

        update_three_pool_wasm_hash(env, new_wasm_hash)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        Admin::require_exist_auth(&env)?;

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
}
