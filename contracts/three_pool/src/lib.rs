#![no_std]

mod pool;
mod token;
mod unit_tests;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

use generic_pool::prelude::*;
use shared::{utils::extend_ttl_instance, Error};
use storage::Admin;

use crate::{pool::ThreePool, token::ThreeToken};

pub const POOL_SIZE: usize = 3;

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
        token_c: Address,
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Result<(), Error> {
        initialize::<POOL_SIZE, ThreePool>(
            env,
            admin,
            a,
            [token_a, token_b, token_c],
            fee_share_bp,
            admin_fee_share_bp,
        )
    }

    pub fn deposit(
        env: Env,
        sender: Address,
        amounts: Vec<u128>,
        min_lp_amount: u128,
    ) -> Result<(), Error> {
        extend_ttl_instance(&env);

        deposit::<POOL_SIZE, ThreePool>(env, sender, amounts, min_lp_amount)
    }

    pub fn withdraw(env: Env, sender: Address, lp_amount: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        withdraw::<POOL_SIZE, ThreePool>(env, sender, lp_amount)
    }

    pub fn swap(
        env: Env,
        sender: Address,
        recipient: Address,
        amount_in: u128,
        receive_amount_min: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<u128, Error> {
        extend_ttl_instance(&env);

        swap::<POOL_SIZE, ThreePool>(
            env,
            sender,
            recipient,
            amount_in,
            receive_amount_min,
            token_from,
            token_to,
        )
    }

    pub fn claim_rewards(env: Env, sender: Address) -> Result<(), Error> {
        extend_ttl_instance(&env);

        claim_rewards::<POOL_SIZE, ThreePool>(env, sender)
    }

    // ----------- Admin -----------

    pub fn claim_admin_fee(env: Env) -> Result<(), Error> {
        extend_ttl_instance(&env);

        claim_admin_fee::<POOL_SIZE, ThreePool>(env)
    }

    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_admin(env, new_admin)
    }

    pub fn set_admin_fee_share(env: Env, admin_fee_share_bp: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_admin_fee_share::<POOL_SIZE, ThreePool>(env, admin_fee_share_bp)
    }

    pub fn set_fee_share(env: Env, fee_share_bp: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_fee_share::<POOL_SIZE, ThreePool>(env, fee_share_bp)
    }

    // ----------- View -----------

    pub fn pending_reward(env: Env, user: Address) -> Result<(u128, u128), Error> {
        pending_reward::<POOL_SIZE, ThreePool>(env, user)
    }

    pub fn get_pool(env: Env) -> Result<ThreePool, Error> {
        get_pool::<POOL_SIZE, ThreePool>(env)
    }

    pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
        get_user_deposit::<POOL_SIZE>(env, user)
    }

    pub fn get_d(env: Env) -> Result<u128, Error> {
        get_d::<POOL_SIZE, ThreePool>(env)
    }

    pub fn get_receive_amount(
        env: Env,
        input: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<(u128, u128), Error> {
        get_receive_amount::<POOL_SIZE, ThreePool>(env, input, token_from, token_to)
    }

    pub fn get_send_amount(
        env: Env,
        output: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<(u128, u128), Error> {
        get_send_amount::<POOL_SIZE, ThreePool>(env, output, token_from, token_to)
    }

    pub fn get_withdraw_amount(env: Env, lp_amount: u128) -> Result<WithdrawAmountView, Error> {
        get_withdraw_amount::<POOL_SIZE, ThreePool>(env, lp_amount)
    }

    pub fn get_deposit_amount(env: Env, amounts: Vec<u128>) -> Result<u128, Error> {
        get_deposit_amount::<POOL_SIZE, ThreePool>(env, amounts)
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        get_admin(env)
    }

    // ----------- Upgrade -----------

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        Admin::require_exist_auth(&env)?;

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
}
