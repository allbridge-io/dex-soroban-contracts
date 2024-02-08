use shared::{utils::extend_ttl_instance, Error};
use soroban_sdk::{contract, contractimpl, Address, Env, Map};

use crate::methods::public::{create_pair, get_pool, get_pools, initialize};

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        initialize(env, admin)
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

    pub fn pool(env: Env, token_a: Address, token_b: Address) -> Result<Address, Error> {
        get_pool(env, &token_a, &token_b)
    }

    pub fn pools(env: Env) -> Result<Map<Address, (Address, Address)>, Error> {
        extend_ttl_instance(&env);

        get_pools(env)
    }
}
