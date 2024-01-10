use shared::{utils::extend_ttl_instance, Error};
use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::{
    methods::{
        internal::pool::Direction,
        public::{claim_rewards, deposit, initialize, swap, withdraw},
        view::{get_pool, get_user_deposit, pending_reward},
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
        a: u128,
        token_a: Address,
        token_b: Address,
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Result<(), Error> {
        initialize(env, a, token_a, token_b, fee_share_bp, admin_fee_share_bp)
    }

    pub fn deposit(env: Env, sender: Address, amount_sp: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        deposit(env, sender, amount_sp)
    }

    pub fn withdraw(env: Env, sender: Address, amount_lp: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        withdraw(env, sender, amount_lp)
    }

    pub fn swap(
        env: Env,
        sender: Address,
        recipient: Address,
        amount_in: u128,
        receive_amount_min: u128,
        zero_fee: bool,
        direction: Direction,
    ) -> Result<u128, Error> {
        extend_ttl_instance(&env);

        swap(
            env,
            sender,
            recipient,
            amount_in,
            receive_amount_min,
            zero_fee,
            direction,
        )
    }

    pub fn claim_rewards(env: Env, sender: Address) -> Result<(), Error> {
        extend_ttl_instance(&env);

        claim_rewards(env, sender)
    }

    /// `view`
    pub fn pending_reward(env: Env, user: Address) -> Result<u128, Error> {
        pending_reward(env, user)
    }

    pub fn get_pool(env: Env) -> Result<Pool, Error> {
        get_pool(env)
    }

    pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
        get_user_deposit(env, user)
    }
}
