use soroban_sdk::{Address, Env, Vec};

use super::{PoolInfo, Token, User, UserDeposit};
use crate::{
    contracts::three_pool::{Client as ThreePoolClient, ThreeToken},
    contracts::two_pool::{Client as TwoPoolClient, TwoToken},
    utils::{
        desoroban_result, float_to_uint, float_to_uint_sp, percentage_to_bp, uint_to_float_sp,
        unwrap_call_result, CallResult,
    },
};

pub trait PoolClient<const N: usize> {
    fn new(env: &Env, id: Address) -> Self;
    fn assert_initialization(
        &self,
        expected_a: u128,
        expected_fee_share_bp: u128,
        expected_admin_fee_share_bp: u128,
    );
    fn deposit(&self, user: &User, deposit_amounts: [f64; N], min_lp_amount: f64);

    fn assert_total_lp_less_or_equal_d(&self);

    fn total_lp(&self) -> u128;

    fn d(&self) -> u128;

    fn user_lp_amount_f64(&self, user: &User) -> f64;

    fn fee_share_bp(&self) -> u128;

    fn admin_fee_share_bp(&self) -> u128;

    fn user_deposit(&self, user: &User) -> UserDeposit;
    fn user_deposit_by_address(&self, user: &Address) -> UserDeposit;

    fn set_admin_fee_share(&self, admin_fee: f64);
    fn set_admin(&self, admin: Address);

    fn set_fee_share(&self, fee_share: f64);

    fn claim_rewards(&self, user: &User);

    fn claim_admin_fee(&self);

    fn withdraw_checked(&self, user: &User, withdraw_amount: f64) -> CallResult;

    fn withdraw(&self, user: &User, withdraw_amount: f64);

    fn deposit_with_address_checked(
        &self,
        user: &Address,
        deposit_amounts: [f64; N],
        min_lp_amount: f64,
    ) -> CallResult;

    fn deposit_with_address(&self, user: &Address, deposit_amounts: [f64; N], min_lp_amount: f64);

    fn pending_reward(&self, user: &User) -> std::vec::Vec<u128>;

    fn deposit_checked(
        &self,
        user: &User,
        deposit_amounts: [f64; N],
        min_lp_amount: f64,
    ) -> CallResult;

    fn swap_checked<T: Into<usize> + Copy>(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        token_from: &Token<T>,
        token_to: &Token<T>,
    ) -> CallResult<u128>;

    fn swap<T: Into<usize> + Copy>(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        token_from: &Token<T>,
        token_to: &Token<T>,
    );

    fn pool_info(&self) -> PoolInfo;
}

#[macro_export]
macro_rules! generate_pool_client {
    ($name:ident, $client_path:ident, $token:tt,  $pool_size:literal) => {
        pub struct $name {
            pub id: soroban_sdk::Address,
            pub client: $client_path<'static>,
            pub env: Env,
        }

        impl PoolClient<$pool_size> for $name {
            fn new(env: &Env, id: Address) -> Self {
                let client = $client_path::new(env, &id);
                $name {
                    id,
                    client,
                    env: env.clone(),
                }
            }

            fn assert_initialization(
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
                assert_eq!(
                    pool_info.token_balances.0,
                    Vec::from_array(&self.env, [0; $pool_size])
                );
                assert_eq!(
                    pool_info.acc_rewards_per_share_p.0,
                    Vec::from_array(&self.env, [0; $pool_size])
                );
                assert_eq!(
                    pool_info.admin_fee_amount.0,
                    Vec::from_array(&self.env, [0; $pool_size])
                );
            }

            fn deposit(&self, user: &User, deposit_amounts: [f64; $pool_size], min_lp_amount: f64) {
                self.deposit_with_address(&user.as_address(), deposit_amounts, min_lp_amount);
            }

            fn pending_reward(&self, user: &User) -> std::vec::Vec<u128> {
                let result = self.client.pending_reward(&user.as_address());
                let len = result.len() as usize;
                let mut vec_result = std::vec::Vec::with_capacity(len);
                vec_result.resize(len, 0);

                for i in 0..len {
                    vec_result[i] = result.get(i as u32).unwrap();
                }

                vec_result
            }

            fn assert_total_lp_less_or_equal_d(&self) {
                let allowed_range = 0..2;
                let total_lp_amount = self.total_lp() as i128;
                let d = self.d() as i128;
                let diff = total_lp_amount.abs_diff(d);

                assert!(
                    allowed_range.contains(&diff),
                    "InvariantFailed: Total lp amount  must be less or equal to D"
                );
            }

            fn total_lp(&self) -> u128 {
                self.client.get_pool().total_lp_amount
            }

            fn d(&self) -> u128 {
                self.client.get_d()
            }

            fn user_lp_amount_f64(&self, user: &User) -> f64 {
                uint_to_float_sp(self.user_deposit(user).lp_amount)
            }

            fn fee_share_bp(&self) -> u128 {
                self.client.get_pool().fee_share_bp
            }

            fn admin_fee_share_bp(&self) -> u128 {
                self.client.get_pool().admin_fee_share_bp
            }

            fn user_deposit(&self, user: &User) -> UserDeposit {
                self.user_deposit_by_address(user.as_ref())
            }

            fn user_deposit_by_address(&self, address: &Address) -> UserDeposit {
                let deposit = self.client.get_user_deposit(address);
                UserDeposit {
                    reward_debts: deposit.reward_debts.0,
                    lp_amount: deposit.lp_amount,
                }
            }

            fn set_admin_fee_share(&self, admin_fee: f64) {
                unwrap_call_result(
                    &self.env,
                    desoroban_result(
                        self.client
                            .try_set_admin_fee_share(&percentage_to_bp(admin_fee)),
                    ),
                );
            }

            fn set_admin(&self, admin: Address) {
                unwrap_call_result(
                    &self.env,
                    desoroban_result(self.client.try_set_admin(&admin)),
                );
            }

            fn set_fee_share(&self, fee_share: f64) {
                unwrap_call_result(
                    &self.env,
                    desoroban_result(self.client.try_set_fee_share(&percentage_to_bp(fee_share))),
                );
            }

            fn claim_rewards(&self, user: &User) {
                unwrap_call_result(
                    &self.env,
                    desoroban_result(self.client.try_claim_rewards(&user.as_address())),
                );
            }

            fn claim_admin_fee(&self) {
                unwrap_call_result(
                    &self.env,
                    desoroban_result(self.client.try_claim_admin_fee()),
                );
            }

            fn withdraw_checked(&self, user: &User, withdraw_amount: f64) -> CallResult {
                desoroban_result(
                    self.client
                        .try_withdraw(&user.as_address(), &float_to_uint_sp(withdraw_amount)),
                )
            }

            fn withdraw(&self, user: &User, withdraw_amount: f64) {
                unwrap_call_result(&self.env, self.withdraw_checked(user, withdraw_amount));
            }

            fn deposit_with_address_checked(
                &self,
                user: &Address,
                deposit_amounts: [f64; $pool_size],
                min_lp_amount: f64,
            ) -> CallResult {
                let mut amounts = Vec::new(&self.env);

                for i in 0usize..$pool_size {
                    amounts.push_back(float_to_uint(deposit_amounts[i], 7));
                }

                desoroban_result(self.client.try_deposit(
                    user,
                    &amounts,
                    &float_to_uint_sp(min_lp_amount),
                ))
            }

            fn deposit_with_address(
                &self,
                user: &Address,
                deposit_amounts: [f64; $pool_size],
                min_lp_amount: f64,
            ) {
                unwrap_call_result(
                    &self.env,
                    self.deposit_with_address_checked(user, deposit_amounts, min_lp_amount),
                );
            }

            fn deposit_checked(
                &self,
                user: &User,
                deposit_amounts: [f64; $pool_size],
                min_lp_amount: f64,
            ) -> CallResult {
                self.deposit_with_address_checked(
                    &user.as_address(),
                    deposit_amounts,
                    min_lp_amount,
                )
            }

            fn swap_checked<T: Into<usize> + Copy>(
                &self,
                sender: &User,
                recipient: &User,
                amount: f64,
                receive_amount_min: f64,
                token_from: &Token<T>,
                token_to: &Token<T>,
            ) -> CallResult<u128> {
                let token_from: usize = token_from.pool_token.into();
                let token_to: usize = token_to.pool_token.into();

                desoroban_result(self.client.try_swap(
                    &sender.as_address(),
                    &recipient.as_address(),
                    &float_to_uint(amount, 7),
                    &float_to_uint(receive_amount_min, 7),
                    &$token::from(token_from),
                    &$token::from(token_to),
                ))
            }

            fn swap<T: Into<usize> + Copy>(
                &self,
                sender: &User,
                recipient: &User,
                amount: f64,
                receive_amount_min: f64,
                token_from: &Token<T>,
                token_to: &Token<T>,
            ) {
                unwrap_call_result(
                    &self.env,
                    self.swap_checked(
                        sender,
                        recipient,
                        amount,
                        receive_amount_min,
                        token_from,
                        token_to,
                    ),
                );
            }

            fn pool_info(&self) -> PoolInfo {
                let info = self.client.get_pool();

                PoolInfo {
                    id: self.id.clone(),
                    d: self.client.get_d(),
                    a: info.a,
                    acc_rewards_per_share_p: info.acc_rewards_per_share_p.0,
                    admin_fee_amount: info.admin_fee_amount.0,
                    admin_fee_share_bp: info.admin_fee_share_bp,
                    fee_share_bp: info.fee_share_bp,
                    token_balances: info.token_balances.0,
                    tokens: info.tokens.0,
                    tokens_decimals: info.tokens_decimals.0,
                    total_lp_amount: info.total_lp_amount,
                }
            }
        }
    };
}

generate_pool_client!(TwoPool, TwoPoolClient, TwoToken, 2);
generate_pool_client!(ThreePool, ThreePoolClient, ThreeToken, 3);
