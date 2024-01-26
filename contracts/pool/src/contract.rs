use shared::{utils::extend_ttl_instance, Error};
use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::{
    methods::{
        internal::pool::Direction,
        public::{claim_admin_fee, claim_rewards, deposit, initialize, swap, withdraw},
        view::{get_d, get_pool, get_user_deposit, pending_reward},
    },
    storage::{pool::Pool, user_deposit::UserDeposit},
};

#[contract]
pub struct PoolContract;

#[contractimpl]
impl PoolContract {
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        env: Env,
        admin: Address,
        a: u128,
        token_a: Address,
        token_b: Address,
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Result<(), Error> {
        initialize(
            env,
            admin,
            a,
            token_a,
            token_b,
            fee_share_bp,
            admin_fee_share_bp,
        )
    }

    pub fn deposit(
        env: Env,
        sender: Address,
        amounts: (u128, u128),
        min_lp_amount: u128,
    ) -> Result<(), Error> {
        extend_ttl_instance(&env);

        deposit(env, sender, amounts, min_lp_amount)
    }

    pub fn withdraw(env: Env, sender: Address, lp_amount: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        withdraw(env, sender, lp_amount)
    }

    pub fn swap(
        env: Env,
        sender: Address,
        recipient: Address,
        amount_in: u128,
        receive_amount_min: u128,
        direction: Direction,
    ) -> Result<u128, Error> {
        extend_ttl_instance(&env);

        swap(
            env,
            sender,
            recipient,
            amount_in,
            receive_amount_min,
            direction,
        )
    }

    pub fn claim_rewards(env: Env, sender: Address) -> Result<(), Error> {
        extend_ttl_instance(&env);

        claim_rewards(env, sender)
    }

    /// `admin`
    pub fn claim_admin_fee(env: Env) -> Result<(), Error> {
        extend_ttl_instance(&env);

        claim_admin_fee(env)
    }

    /// `view`
    pub fn pending_reward(env: Env, user: Address) -> Result<(u128, u128), Error> {
        pending_reward(env, user)
    }

    pub fn get_pool(env: Env) -> Result<Pool, Error> {
        get_pool(env)
    }

    pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
        get_user_deposit(env, user)
    }

    pub fn get_d(env: Env) -> Result<u128, Error> {
        get_d(env)
    }
}
