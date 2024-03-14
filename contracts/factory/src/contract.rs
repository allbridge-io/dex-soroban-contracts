use shared::{utils::extend_ttl_instance, Error};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map};
use storage::Admin;

use crate::methods::public::{
    create_pair, get_admin, get_pool, get_pools, get_wasm_hash, initialize, set_admin,
    update_wasm_hash,
};

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    pub fn initialize(env: Env, wasm_hash: BytesN<32>, admin: Address) -> Result<(), Error> {
        initialize(env, wasm_hash, admin)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_pair(
        env: Env,
        deployer: Address,
        pool_admin: Address,
        a: u128,
        token_a: Address,
        token_b: Address,
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Result<Address, Error> {
        extend_ttl_instance(&env);

        create_pair(
            env,
            deployer,
            pool_admin,
            a,
            token_a,
            token_b,
            fee_share_bp,
            admin_fee_share_bp,
        )
    }

    // ----------- Admin -----------

    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_admin(env, new_admin)
    }

    // ----------- View -----------

    pub fn pool(env: Env, token_a: Address, token_b: Address) -> Result<Address, Error> {
        extend_ttl_instance(&env);

        get_pool(env, &token_a, &token_b)
    }

    pub fn pools(env: Env) -> Result<Map<Address, (Address, Address)>, Error> {
        extend_ttl_instance(&env);

        get_pools(env)
    }

    pub fn get_wasm_hash(env: Env) -> Result<BytesN<32>, Error> {
        get_wasm_hash(env)
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        get_admin(env)
    }

    // ----------- Upgrade -----------

    pub fn update_wasm_hash(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        extend_ttl_instance(&env);

        update_wasm_hash(env, new_wasm_hash)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        Admin::require_exist_auth(&env)?;

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
}
