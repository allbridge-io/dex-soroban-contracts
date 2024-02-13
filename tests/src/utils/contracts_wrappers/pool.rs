use std::cmp::Ordering;

use ethnum::U256;
use soroban_sdk::{Address, Env};

use crate::{
    contracts::pool::{self, Direction, UserDeposit},
    utils::{
        desoroban_result, float_to_int_sp, float_to_uint, int_to_float_sp, uint_to_float,
        CallResult, SYSTEM_PRECISION,
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
}

impl Pool {
    pub const BP: u128 = 10000;

    pub fn new(env: &Env, id: Address) -> Pool {
        let client = pool::Client::new(env, &id);
        Pool { id, client }
    }

    pub fn get_y(&self, native_x: u128, d: u128) -> u128 {
        let a = self.client.get_pool().a;

        let a4 = a << 2;
        let ddd = U256::new(d * d) * d;
        // 4A(D - x) - D
        let part1 = a4 as i128 * (d as i128 - native_x as i128) - d as i128;
        // x * (4AD³ + x(part1²))
        let part2 = (ddd * a4 + (U256::new((part1 * part1) as u128) * native_x)) * native_x;
        // (sqrt(part2) + x(part1)) / 8Ax)
        (sqrt(&part2).as_u128() as i128 + (native_x as i128 * part1)) as u128
            / ((a << 3) * native_x)
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

    pub fn get_lp_amount(&self, d0: u128) -> f64 {
        let lp_amount = self.amount_from_system_precision(self.d() - d0, 7);
        uint_to_float(lp_amount, 7)
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

    // TODO: Call it assert_...
    pub fn invariant_total_lp_less_or_equal_d(&self) {
        let allowed_range = 0..2;
        let total_lp_amount = self.total_lp() as i128;
        let d = self.d() as i128;
        let diff = total_lp_amount.abs_diff(d);

        assert!(
            allowed_range.contains(&diff),
            "InvariantFailed: Total lp amount  must be less or equal to D"
        );
    }

    pub fn user_lp_amount(&self, user: &User) -> u128 {
        self.user_deposit(user).lp_amount
    }

    pub fn user_lp_amount_f64(&self, user: &User) -> f64 {
        int_to_float_sp(self.user_deposit(user).lp_amount)
    }

    pub fn withdraw_amounts(&self, user: &User) -> (f64, f64) {
        let user_lp_amount = self.user_lp_amount(user);
        let token_a_amount = self.token_a_balance() * user_lp_amount / self.total_lp_amount();
        let token_b_amount = self.token_b_balance() * user_lp_amount / self.total_lp_amount();

        (
            int_to_float_sp(token_a_amount),
            int_to_float_sp(token_b_amount),
        )
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

    pub fn claim_rewards(&self, user: &User) -> CallResult {
        desoroban_result(self.client.try_claim_rewards(&user.as_address()))
    }

    pub fn claim_admin_fee(&self) -> CallResult {
        desoroban_result(self.client.try_claim_admin_fee())
    }

    pub fn withdraw(&self, user: &User, withdraw_amount: f64) -> CallResult {
        desoroban_result(
            self.client
                .try_withdraw(&user.as_address(), &float_to_int_sp(withdraw_amount)),
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
        desoroban_result(self.client.try_deposit(
            user,
            &(
                float_to_uint(deposit_amounts.0, 7),
                float_to_uint(deposit_amounts.1, 7),
            ),
            &float_to_int_sp(min_lp_amount),
        ))
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
    ) -> CallResult<u128> {
        desoroban_result(self.client.try_swap(
            &sender.as_address(),
            &recipient.as_address(),
            &float_to_uint(amount, 7),
            &float_to_uint(receive_amount_min, 7),
            &direction,
        ))
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
