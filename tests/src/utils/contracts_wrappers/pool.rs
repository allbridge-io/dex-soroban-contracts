use soroban_sdk::{Address, Env};

use crate::{
    contracts::pool::{self, Direction, UserDeposit},
    utils::{desoroban_result, float_to_int, int_to_float, CallResult},
};

use super::User;

pub struct Pool {
    pub id: soroban_sdk::Address,
    pub client: pool::Client<'static>,
}

impl Pool {
    pub fn create(
        env: &Env,
        a: u128,
        token_a: &Address,
        token_b: &Address,
        fee_share_bp: u128,
        admin_fee: u128,
    ) -> Pool {
        let id = env.register_contract_wasm(None, pool::WASM);
        let client = pool::Client::new(&env, &id);

        client.initialize(&a, &token_a, &token_b, &fee_share_bp, &admin_fee);

        Pool { id, client }
    }

    pub fn user_lp_amount(&self, user: &User) -> u128 {
        self.user_deposit(user).lp_amount
    }

    pub fn withdraw_amounts(&self, user: &User) -> (f64, f64) {
        let user_lp_amount = self.user_lp_amount(user);
        let token_a_amount = self.token_a_balance() * user_lp_amount / self.total_lp_amount();
        let token_b_amount = self.token_b_balance() * user_lp_amount / self.total_lp_amount();

        (int_to_float(token_a_amount), int_to_float(token_b_amount))
    }

    pub fn d(&self) -> u128 {
        self.client.get_pool().d
    }

    pub fn token_a_balance(&self) -> u128 {
        self.client.get_pool().token_a_balance
    }

    pub fn token_b_balance(&self) -> u128 {
        self.client.get_pool().token_b_balance
    }

    pub fn total_lp_amount(&self) -> u128 {
        self.client.get_pool().total_lp_amount
    }

    pub fn acc_reward_a_per_share_p(&self) -> u128 {
        self.client.get_pool().acc_reward_a_per_share_p
    }

    pub fn acc_reward_b_per_share_p(&self) -> u128 {
        self.client.get_pool().acc_reward_b_per_share_p
    }

    pub fn fee_share_bp(&self) -> u128 {
        self.client.get_pool().fee_share_bp
    }

    pub fn admin_fee_share_bp(&self) -> u128 {
        self.client.get_pool().admin_fee_share_bp
    }

    pub fn user_deposit(&self, user: &User) -> UserDeposit {
        self.user_deposit_by_id(&user.as_address())
    }

    pub fn user_deposit_by_id(&self, id: &Address) -> UserDeposit {
        self.client.get_user_deposit(&id)
    }

    pub fn claim_rewards(&self, user: &User) -> CallResult {
        desoroban_result(self.client.try_claim_rewards(&user.as_address()))
    }

    pub fn withdraw(&self, user: &User, withdraw_amount: f64) -> CallResult {
        desoroban_result(
            self.client
                .try_withdraw(&user.as_address(), &float_to_int(withdraw_amount)),
        )
    }

    pub fn withdraw_raw(&self, user: &User, withdraw_amount: u128) -> CallResult {
        desoroban_result(
            self.client
                .try_withdraw(&user.as_address(), &withdraw_amount),
        )
    }

    /// (yusd, yaro)
    pub fn deposit_by_id(
        &self,
        user: &Address,
        deposit_amounts: (f64, f64),
        min_lp_amount: f64,
    ) -> CallResult {
        self.client
            .try_deposit(
                &user,
                &(
                    float_to_int(deposit_amounts.0),
                    float_to_int(deposit_amounts.1),
                ),
                &float_to_int(min_lp_amount),
            )
            .map(Result::unwrap)
            .map_err(Result::unwrap)
    }

    /// (yusd, yaro)
    pub fn deposit(
        &self,
        user: &User,
        deposit_amounts: (f64, f64),
        min_lp_amount: f64,
    ) -> CallResult {
        self.deposit_by_id(&user.as_address(), deposit_amounts, min_lp_amount)
    }

    pub fn swap(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        direction: Direction,
    ) -> u128 {
        self.client.swap(
            &sender.as_address(),
            &recipient.as_address(),
            &float_to_int(amount),
            &float_to_int(receive_amount_min),
            &false,
            &direction,
        )
    }
}
