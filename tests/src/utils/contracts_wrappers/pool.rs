use std::cmp::Ordering;

use ethnum::U256;
use soroban_sdk::{Address, Env};

use crate::{
    contracts::pool::{self, Direction, UserDeposit},
    utils::{
        desoroban_result, float_to_uint, float_to_uint_sp, uint_to_float_sp, unwrap_call_result,
        CallResult, Snapshot, TestingEnv, SYSTEM_PRECISION,
    },
};

use super::User;

pub fn sqrt(n: &U256) -> U256 {
    if *n == U256::ZERO {
        return U256::ZERO;
    }
    let shift: u32 = (255 - n.leading_zeros()) & !1;
    let mut bit = U256::ONE << shift;

    let mut n = *n;
    let mut result = U256::ZERO;
    for _ in (0..shift + 1).step_by(2) {
        let res_bit = result + bit;
        result >>= 1;
        if n >= res_bit {
            n -= res_bit;
            result += bit;
        }
        bit >>= 2;
    }
    result
}

pub struct Pool {
    pub id: soroban_sdk::Address,
    pub client: pool::Client<'static>,
    pub env: Env,
}

impl Pool {
    pub const BP: u128 = 10000;

    pub fn new(env: &Env, id: Address) -> Pool {
        let client = pool::Client::new(env, &id);
        Pool {
            id,
            client,
            env: env.clone(),
        }
    }

    pub fn receive_amount(&self, amount: f64, directin: Direction) -> (u128, u128) {
        self.client.get_receive_amount(
            &float_to_uint(amount, 7),
            &(match directin {
                Direction::A2B => pool::Token::A,
                Direction::B2A => pool::Token::B,
            }),
        )
    }

    pub fn assert_initialization(
        &self,
        expected_a: u128,
        expected_fee_share_bp: u128,
        expected_admin_fee_share_bp: u128,
    ) {
        let pool_info = self.client.get_pool();

        assert_eq!(pool_info.a, expected_a);
        assert_eq!(pool_info.fee_share_bp, expected_fee_share_bp);
        assert_eq!(pool_info.admin_fee_share_bp, expected_admin_fee_share_bp);

        assert_eq!(pool_info.total_lp_amount, 0);
        assert_eq!(pool_info.token_balances.data, (0, 0));
        assert_eq!(pool_info.acc_rewards_per_share_p.data, (0, 0));
        assert_eq!(pool_info.admin_fee_amount.data, (0, 0));
    }

    pub fn total_lp(&self) -> u128 {
        self.client.get_pool().total_lp_amount
    }

    pub fn d(&self) -> u128 {
        self.client.get_d()
    }

    pub fn convert_to_bp(v: f64) -> u128 {
        (v * 10_000.0) as u128
    }

    pub fn set_admin_fee_share(&self, admin_fee: f64) {
        unwrap_call_result(
            &self.env,
            desoroban_result(
                self.client
                    .try_set_admin_fee_share(&Pool::convert_to_bp(admin_fee)),
            ),
        )
    }

    pub fn set_fee_share(&self, fee_share: f64) {
        unwrap_call_result(
            &self.env,
            desoroban_result(
                self.client
                    .try_set_fee_share(&Pool::convert_to_bp(fee_share)),
            ),
        )
    }

    pub fn assert_total_lp_less_or_equal_d(&self) {
        let allowed_range = 0..2;
        let total_lp_amount = self.total_lp() as i128;
        let d = self.d() as i128;
        let diff = total_lp_amount.abs_diff(d);

        assert!(
            allowed_range.contains(&diff),
            "InvariantFailed: Total lp amount  must be less or equal to D"
        );
    }

    pub fn user_lp_amount_f64(&self, user: &User) -> f64 {
        uint_to_float_sp(self.user_deposit(user).lp_amount)
    }

    pub fn token_a_balance(&self) -> u128 {
        self.client.get_pool().token_balances.data.0
    }

    pub fn token_b_balance(&self) -> u128 {
        self.client.get_pool().token_balances.data.1
    }

    pub fn total_lp_amount(&self) -> u128 {
        self.client.get_pool().total_lp_amount
    }

    pub fn acc_reward_a_per_share_p(&self) -> u128 {
        self.client.get_pool().acc_rewards_per_share_p.data.0
    }

    pub fn acc_reward_b_per_share_p(&self) -> u128 {
        self.client.get_pool().acc_rewards_per_share_p.data.1
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
        self.client.get_user_deposit(id)
    }

    pub fn claim_rewards(&self, user: &User) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_claim_rewards(&user.as_address())),
        )
    }

    pub fn claim_rewards_with_snapshots(
        &self,
        testing_env: &TestingEnv,
        user: &User,
    ) -> (Snapshot, Snapshot) {
        let snapshot_before = Snapshot::take(testing_env);
        self.claim_rewards(user);
        let snapshot_after = Snapshot::take(testing_env);
        snapshot_before.print_change_with(&snapshot_after, "Alice claim rewards");

        (snapshot_before, snapshot_after)
    }

    pub fn claim_admin_fee(&self) {
        unwrap_call_result(
            &self.env,
            desoroban_result(self.client.try_claim_admin_fee()),
        )
    }

    pub fn claim_admin_fee_with_snapshots(&self, testing_env: &TestingEnv) -> (Snapshot, Snapshot) {
        let snapshot_before = Snapshot::take(testing_env);
        self.claim_admin_fee();
        let snapshot_after = Snapshot::take(testing_env);
        snapshot_before.print_change_with(&snapshot_after, "Alice claim rewards");

        (snapshot_before, snapshot_after)
    }

    pub fn withdraw_checked(&self, user: &User, withdraw_amount: f64) -> CallResult {
        desoroban_result(
            self.client
                .try_withdraw(&user.as_address(), &float_to_uint_sp(withdraw_amount)),
        )
    }

    pub fn withdraw(&self, user: &User, withdraw_amount: f64) {
        unwrap_call_result(&self.env, self.withdraw_checked(user, withdraw_amount))
    }

    pub fn withdraw_with_snapshots(
        &self,
        testing_env: &TestingEnv,
        user: &User,
        withdraw_amount: f64,
    ) -> (Snapshot, Snapshot) {
        let snapshot_before = Snapshot::take(testing_env);
        self.withdraw(user, withdraw_amount);
        let snapshot_after = Snapshot::take(testing_env);
        snapshot_before.print_change_with(&snapshot_after, "Alice claim rewards");

        (snapshot_before, snapshot_after)
    }

    /// (yusd, yaro)
    pub fn deposit_with_address_checked(
        &self,
        user: &Address,
        deposit_amounts: (f64, f64),
        min_lp_amount: f64,
    ) -> CallResult {
        desoroban_result(self.client.try_deposit(
            user,
            &(
                float_to_uint(deposit_amounts.0, 7),
                float_to_uint(deposit_amounts.1, 7),
            ),
            &float_to_uint_sp(min_lp_amount),
        ))
    }

    /// (yusd, yaro)
    pub fn deposit_with_address(
        &self,
        user: &Address,
        deposit_amounts: (f64, f64),
        min_lp_amount: f64,
    ) {
        unwrap_call_result(
            &self.env,
            self.deposit_with_address_checked(user, deposit_amounts, min_lp_amount),
        )
    }

    /// (yusd, yaro)
    pub fn deposit_checked(
        &self,
        user: &User,
        deposit_amounts: (f64, f64),
        min_lp_amount: f64,
    ) -> CallResult {
        self.deposit_with_address_checked(&user.as_address(), deposit_amounts, min_lp_amount)
    }

    /// (yusd, yaro)
    pub fn deposit(&self, user: &User, deposit_amounts: (f64, f64), min_lp_amount: f64) {
        self.deposit_with_address(&user.as_address(), deposit_amounts, min_lp_amount)
    }

    /// (yusd, yaro)
    pub fn deposit_with_snapshots(
        &self,
        testing_env: &TestingEnv,
        user: &User,
        deposit_amounts: (f64, f64),
        min_lp_amount: f64,
    ) -> (Snapshot, Snapshot) {
        let snapshot_before = Snapshot::take(testing_env);
        self.deposit_with_address(&user.as_address(), deposit_amounts, min_lp_amount);
        let snapshot_after = Snapshot::take(testing_env);

        (snapshot_before, snapshot_after)
    }

    pub fn swap_checked(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        direction: Direction,
    ) -> CallResult<u128> {
        desoroban_result(self.client.try_swap(
            &sender.as_address(),
            &recipient.as_address(),
            &float_to_uint(amount, 7),
            &float_to_uint(receive_amount_min, 7),
            &direction,
        ))
    }

    pub fn swap(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        direction: Direction,
    ) {
        unwrap_call_result(
            &self.env,
            self.swap_checked(sender, recipient, amount, receive_amount_min, direction),
        );
    }

    pub fn amount_to_system_precision(&self, amount: u128, decimals: u32) -> u128 {
        match decimals.cmp(&SYSTEM_PRECISION) {
            Ordering::Greater => amount / (10u128.pow(decimals - SYSTEM_PRECISION)),
            Ordering::Less => amount * (10u128.pow(SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }

    pub fn amount_from_system_precision(&self, amount: u128, decimals: u32) -> u128 {
        match decimals.cmp(&SYSTEM_PRECISION) {
            Ordering::Greater => amount * (10u128.pow(decimals - SYSTEM_PRECISION)),
            Ordering::Less => amount / (10u128.pow(SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }
}
