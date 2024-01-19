use shared::Error;
use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::methods::public::{create_pair, get_pool, initialize};

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    pub fn initialize(env: Env) -> Result<(), Error> {
        initialize(env)
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

    pub fn get_pool(env: Env, token_a: Address, token_b: Address) -> Result<Address, Error> {
        get_pool(&env, &token_a, &token_b)
    }
}
