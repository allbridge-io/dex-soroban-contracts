#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use generic_pool::{
    methods::{public::*, view::*},
    storage::user_deposit::UserDeposit,
    three_pool::{pool_view::WithdrawAmountView, three_pool::ThreePool, token::ThreeToken},
};
use shared::{utils::extend_ttl_instance, Error};
use storage::Admin;

const SIZE: usize = 3;

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
        initialize::<SIZE, ThreePool>(
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
        amounts: (u128, u128, u128),
        min_lp_amount: u128,
    ) -> Result<(), Error> {
        extend_ttl_instance(&env);

        deposit::<SIZE, ThreePool>(
            env,
            sender,
            [amounts.0, amounts.1, amounts.2],
            min_lp_amount,
        )
    }

    pub fn withdraw(env: Env, sender: Address, lp_amount: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        withdraw::<SIZE, ThreePool>(env, sender, lp_amount)
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

        swap::<SIZE, ThreePool>(
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

        claim_rewards::<SIZE, ThreePool>(env, sender)
    }

    // ----------- Admin -----------

    pub fn claim_admin_fee(env: Env) -> Result<(), Error> {
        extend_ttl_instance(&env);

        claim_admin_fee::<SIZE, ThreePool>(env)
    }

    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_admin(env, new_admin)
    }

    pub fn set_admin_fee_share(env: Env, admin_fee_share_bp: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_admin_fee_share::<SIZE, ThreePool>(env, admin_fee_share_bp)
    }

    pub fn set_fee_share(env: Env, fee_share_bp: u128) -> Result<(), Error> {
        extend_ttl_instance(&env);

        set_fee_share::<SIZE, ThreePool>(env, fee_share_bp)
    }

    // ----------- View -----------

    pub fn pending_reward(env: Env, user: Address) -> Result<(u128, u128), Error> {
        pending_reward::<SIZE, ThreePool>(env, user)
    }

    pub fn get_pool(env: Env) -> Result<ThreePool, Error> {
        get_pool::<SIZE, ThreePool>(env)
    }

    pub fn get_user_deposit(env: Env, user: Address) -> Result<UserDeposit, Error> {
        get_user_deposit::<SIZE>(env, user)
    }

    pub fn get_d(env: Env) -> Result<u128, Error> {
        get_d::<SIZE, ThreePool>(env)
    }

    pub fn get_receive_amount(
        env: Env,
        input: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<(u128, u128), Error> {
        get_receive_amount::<SIZE, ThreePool>(env, input, token_from, token_to)
    }

    pub fn get_send_amount(
        env: Env,
        output: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<(u128, u128), Error> {
        get_send_amount::<SIZE, ThreePool>(env, output, token_from, token_to)
    }

    pub fn get_withdraw_amount(env: Env, lp_amount: u128) -> Result<WithdrawAmountView, Error> {
        get_withdraw_amount::<SIZE, ThreePool, WithdrawAmountView>(env, lp_amount)
    }

    pub fn get_deposit_amount(env: Env, amounts: (u128, u128, u128)) -> Result<u128, Error> {
        get_deposit_amount::<SIZE, ThreePool>(env, amounts)
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
