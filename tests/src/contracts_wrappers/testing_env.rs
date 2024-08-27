use std::ops::Index;

use soroban_sdk::Address;

use crate::utils::{assert_rel_eq, float_to_uint, float_to_uint_sp, floats_to_uint};

use super::{EventAsserts, Token, User};

pub struct UserBalance<const N: usize> {
    pub balances: [u128; N],
    pub lp_amount: u128,
}

#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub id: Address,
    pub d: u128,
    pub a: u128,
    pub acc_rewards_per_share_p: soroban_sdk::Vec<u128>,
    pub admin_fee_amount: soroban_sdk::Vec<u128>,
    pub admin_fee_share_bp: u128,
    pub fee_share_bp: u128,
    pub token_balances: soroban_sdk::Vec<u128>,
    pub tokens: soroban_sdk::Vec<Address>,
    pub tokens_decimals: soroban_sdk::Vec<u32>,
    pub total_lp_amount: u128,
}

#[derive(Debug, Clone)]
pub struct UserDeposit {
    pub lp_amount: u128,
    pub reward_debts: soroban_sdk::Vec<u128>,
}

pub trait Snapshot<const N: usize>:
    Index<String, Output = u128> + Index<&'static str, Output = u128> + Clone
{
    type TestingEnv: TestingEnv<N>;

    fn get_user_balances(&self, user: &User) -> UserBalance<N>;
    fn take(testing_env: &impl TestingEnv<N>) -> Self;
    fn print_change_with(&self, other: &Self, title: &str);

    fn get_user_balances_sum(&self, user: &User) -> u128 {
        let user_balance = self.get_user_balances(user);
        user_balance.balances.iter().sum::<u128>()
    }
}

pub trait TestingEnv<const N: usize>: Sized {
    type Snapshot: Snapshot<N>;

    const TOKENS: [&'static str; N];

    fn event_asserts(&self) -> &EventAsserts<N>;
    fn assert_total_lp_less_or_equal_d(&self);

    fn withdraw(&self, user: &User, withdraw_amount: f64);
    fn deposit(&self, user: &User, deposit_amounts: [f64; N], min_lp_amount: f64);
    fn claim_rewards(&self, user: &User);
    fn claim_admin_fee(&self);
    fn swap<T: Into<usize> + Copy>(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        token_from: &Token<T>,
        token_to: &Token<T>,
    );

    fn users(&self) -> (&User, &User, &User);
    fn tokens(&self) -> [&Token<impl Into<usize>>; N];

    fn pool_info(&self) -> PoolInfo;
    fn get_user_deposit(&self, user_address: &Address) -> UserDeposit;

    /* -------- Asserts ----------- */

    fn assert_claim_admin_fee(
        snapshot_before: &Self::Snapshot,
        snapshot_after: &Self::Snapshot,
        rewards: [f64; N],
    ) {
        let rewards = floats_to_uint(rewards, 7);

        let diffs = Self::TOKENS
            .iter()
            .map(|token| {
                let admin_balance_row = format!("admin_{token}_balance");
                let admin_diff =
                    snapshot_after[admin_balance_row.clone()] - snapshot_before[admin_balance_row];

                let pool_balance_row = format!("pool_{token}_balance");
                let pool_diff =
                    snapshot_before[pool_balance_row.clone()] - snapshot_after[pool_balance_row];

                (admin_diff, pool_diff)
            })
            .collect::<Vec<_>>();

        for (index, reward) in rewards.into_iter().enumerate() {
            let (admin_diff, pool_diff) = diffs[index];

            assert_rel_eq(admin_diff, reward, 2);
            assert_rel_eq(pool_diff, reward, 2);
        }
    }

    fn assert_withdraw_balances(
        snapshot_before: &Self::Snapshot,
        snapshot_after: &Self::Snapshot,

        expected_amounts: [f64; N],
        expected_fees: [f64; N],
        expected_admin_fees: [f64; N],
        expected_rewards: [f64; N],

        user_before: [u128; N],
        user_after: [u128; N],
    ) {
        let check_fees = expected_fees.iter().sum::<f64>() != 0.0;

        let user_diffs: [u128; N] = core::array::from_fn(|i| user_after[i] - user_before[i]);

        let expected_diffs: [u128; N] = floats_to_uint(
            core::array::from_fn(|i| expected_amounts[i] + expected_rewards[i]),
            7,
        );
        let expected_admin_fees = floats_to_uint(expected_admin_fees, 7);

        for (index, token) in Self::TOKENS.iter().enumerate() {
            let pool_balance_key = format!("pool_{token}_balance");
            let pool_diff = snapshot_before[pool_balance_key.clone()]
                - snapshot_after[pool_balance_key.clone()];

            let admin_fee_reward_key = format!("admin_{token}_fee_rewards");
            let admin_fee_diff = snapshot_after[admin_fee_reward_key.clone()]
                - snapshot_before[admin_fee_reward_key.clone()];

            assert_eq!(admin_fee_diff, expected_admin_fees[index]);

            assert_rel_eq(user_diffs[index], expected_diffs[index], 1);
            assert_rel_eq(pool_diff, expected_diffs[index], 1);

            if check_fees {
                let reward_share_key = format!("acc_reward_{token}_per_share_p");
                assert!(
                    snapshot_before[reward_share_key.clone()]
                        < snapshot_after[reward_share_key.clone()]
                );
            }
        }
    }

    fn assert_withdraw(
        &self,
        snapshot_before: Self::Snapshot,
        snapshot_after: Self::Snapshot,
        user: &User,
        expected_amounts: [f64; N],
        expected_fees: [f64; N],
        expected_rewards: [f64; N],
        expected_user_withdraw_lp_diff: f64,
        expected_admin_fees: [f64; N],
    ) {
        self.assert_total_lp_less_or_equal_d();
        self.event_asserts().assert_withdraw_event(
            user,
            expected_user_withdraw_lp_diff,
            expected_amounts,
            expected_fees,
        );

        let UserBalance {
            balances: user_before,
            lp_amount: user_lp_amount_before,
        } = snapshot_before.get_user_balances(user);
        let UserBalance {
            balances: user_after,
            lp_amount: user_lp_amount_after,
        } = snapshot_after.get_user_balances(user);

        Self::assert_withdraw_balances(
            &snapshot_before,
            &snapshot_after,
            expected_amounts,
            expected_fees,
            expected_admin_fees,
            expected_rewards,
            user_before,
            user_after,
        );

        let user_lp_diff = user_lp_amount_before - user_lp_amount_after;
        let pool_lp_amount_diff =
            snapshot_before["total_lp_amount"] - snapshot_after["total_lp_amount"];
        let expected_user_withdraw_lp_amount = float_to_uint_sp(expected_user_withdraw_lp_diff);

        assert!(snapshot_before["total_lp_amount"] > snapshot_after["total_lp_amount"]);
        assert!(snapshot_before["d"] > snapshot_after["d"]);
        assert_eq!(user_lp_diff, pool_lp_amount_diff);
        assert_eq!(user_lp_diff, expected_user_withdraw_lp_amount);
        assert_eq!(pool_lp_amount_diff, expected_user_withdraw_lp_amount);
    }

    fn assert_claim(
        &self,
        snapshot_before: Self::Snapshot,
        snapshot_after: Self::Snapshot,
        user: &User,
        rewards: [f64; N],
    ) {
        self.assert_total_lp_less_or_equal_d();
        if rewards.iter().sum::<f64>() != 0.0 {
            self.event_asserts()
                .assert_claimed_reward_event(user, rewards);
        }

        let UserBalance {
            balances: user_before,
            ..
        } = snapshot_before.get_user_balances(user);
        let UserBalance {
            balances: user_after,
            ..
        } = snapshot_after.get_user_balances(user);

        let user_diffs: [u128; N] = core::array::from_fn(|i| user_after[i] - user_before[i]);
        let rewards = floats_to_uint(rewards, 7);

        for (i, token) in Self::TOKENS.iter().enumerate() {
            let pool_balance_key = format!("pool_{token}_balance");
            let pool_diff = snapshot_before[pool_balance_key.clone()]
                - snapshot_after[pool_balance_key.clone()];

            assert_eq!(user_diffs[i], rewards[i]);
            assert_eq!(pool_diff, rewards[i]);
        }
    }

    fn assert_swap(
        &self,
        snapshot_before: Self::Snapshot,
        snapshot_after: Self::Snapshot,
        sender: &User,
        recipient: &User,
        token_from: &Token<impl Into<usize>>,
        token_to: &Token<impl Into<usize>>,
        amount: f64,
        expected_receive_amount: f64,
        expected_fee: f64,
    ) {
        self.assert_total_lp_less_or_equal_d();

        self.event_asserts().assert_swapped_event(
            sender,
            recipient,
            token_from,
            token_to,
            amount,
            expected_receive_amount,
            expected_fee,
        );

        let sender_tag = sender.tag;
        let recipient_tag = recipient.tag;

        let (from_token_tag, to_token_tag) = (token_from.tag.clone(), token_to.tag.clone());

        let sender_balance_key = format!("{sender_tag}_{from_token_tag}_balance");
        let recipient_balance_key = format!("{recipient_tag}_{to_token_tag}_balance");
        let pool_from_balance_key = format!("pool_{from_token_tag}_balance");
        let pool_to_balance_key = format!("pool_{to_token_tag}_balance");
        let acc_reward_token_to_per_share_p_key = format!("acc_reward_{to_token_tag}_per_share_p");

        let expected_receive_amount = float_to_uint(expected_receive_amount, 7);
        let _expected_fee = float_to_uint(expected_fee, 7);
        let amount = float_to_uint(amount, 7);

        let sender_from_token_diff =
            snapshot_before[sender_balance_key.clone()] - snapshot_after[sender_balance_key];

        let recipient_to_token_diff =
            snapshot_after[recipient_balance_key.clone()] - snapshot_before[recipient_balance_key];

        let pool_from_token_diff =
            snapshot_after[pool_from_balance_key.clone()] - snapshot_before[pool_from_balance_key];
        let pool_to_token_diff =
            snapshot_before[pool_to_balance_key.clone()] - snapshot_after[pool_to_balance_key];

        assert!(
            snapshot_after[acc_reward_token_to_per_share_p_key.clone()]
                > snapshot_before[acc_reward_token_to_per_share_p_key]
        );

        assert_eq!(recipient_to_token_diff, expected_receive_amount);
        assert_eq!(pool_to_token_diff, expected_receive_amount);

        assert_eq!(pool_to_token_diff, expected_receive_amount);
        assert_eq!(recipient_to_token_diff, expected_receive_amount);

        assert_eq!(sender_from_token_diff, amount);
        assert_eq!(pool_from_token_diff, amount);
    }

    fn assert_deposit(
        &self,
        snapshot_before: Self::Snapshot,
        snapshot_after: Self::Snapshot,
        user: &User,
        expected_deposits: [f64; N],
        expected_rewards: [f64; N],
        expected_lp_amount: f64,
    ) {
        self.event_asserts()
            .assert_deposit_event(user, expected_lp_amount, expected_deposits);
        self.assert_deposit_without_event(
            snapshot_before,
            snapshot_after,
            user,
            expected_deposits,
            expected_rewards,
            expected_lp_amount,
        );
    }

    fn assert_deposit_without_event(
        &self,
        snapshot_before: Self::Snapshot,
        snapshot_after: Self::Snapshot,
        user: &User,
        deposits: [f64; N],
        expected_rewards: [f64; N],
        expected_lp_amount: f64,
    ) {
        self.assert_total_lp_less_or_equal_d();

        let UserBalance {
            balances: user_before,
            lp_amount: user_lp_amount_before,
        } = snapshot_before.get_user_balances(user);
        let UserBalance {
            balances: user_after,
            lp_amount: user_lp_amount_after,
        } = snapshot_after.get_user_balances(user);

        let deposits = floats_to_uint(deposits, 7);
        let expected_rewards = floats_to_uint(expected_rewards, 7);

        let expected_lp_amount = float_to_uint_sp(expected_lp_amount);

        let user_lp_diff = user_lp_amount_after - user_lp_amount_before;

        let user_diff: [u128; N] =
            core::array::from_fn(|i| deposits[i] - (user_before[i] - user_after[i]));

        let pool_diff: [u128; N] = core::array::from_fn(|i| {
            let pool_balance_key = format!("pool_{}_balance", Self::TOKENS[i]);

            deposits[i]
                - (snapshot_after[pool_balance_key.clone()] - snapshot_before[pool_balance_key])
        });

        assert!(snapshot_before["total_lp_amount"] < snapshot_after["total_lp_amount"]);
        assert_eq!(
            snapshot_after["total_lp_amount"] - snapshot_before["total_lp_amount"],
            user_lp_diff
        );
        assert!(snapshot_before["d"] < snapshot_after["d"]);
        assert_eq!(user_lp_diff, expected_lp_amount);

        for i in 0..N {
            assert_eq!(user_diff[i], expected_rewards[i]);
            assert_eq!(pool_diff[i], expected_rewards[i]);
        }
    }

    /* -------- Methods ----------- */

    fn do_claim(&self, user: &User, expected_rewards: [f64; N]) {
        let snapshot_before = Self::Snapshot::take(self);
        self.claim_rewards(user);
        let snapshot_after = Self::Snapshot::take(self);

        let title = format!("Claim rewards, expected {:?}", expected_rewards);
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_claim(snapshot_before, snapshot_after, user, expected_rewards);
    }

    fn do_withdraw(
        &self,
        user: &User,
        withdraw_amount: f64,
        expected_withdraw_amounts: [f64; N],
        expected_fee: [f64; N],
        expected_rewards: [f64; N],
        expected_user_lp_diff: f64,
        expected_admin_fee: [f64; N],
    ) -> (Self::Snapshot, Self::Snapshot) {
        let snapshot_before = Self::Snapshot::take(self);
        self.withdraw(user, withdraw_amount);
        let snapshot_after = Self::Snapshot::take(self);
        snapshot_before.print_change_with(&snapshot_after, "Withdraw");

        if expected_rewards.iter().sum::<f64>() != 0.0 {
            self.event_asserts()
                .assert_claimed_reward_event(user, expected_rewards);
        }

        self.assert_withdraw(
            snapshot_before.clone(),
            snapshot_after.clone(),
            user,
            expected_withdraw_amounts,
            expected_fee,
            expected_rewards,
            expected_user_lp_diff,
            expected_admin_fee,
        );

        (snapshot_before, snapshot_after)
    }

    fn do_deposit(
        &self,
        user: &User,
        deposit: [f64; N],
        expected_rewards: [f64; N],
        expected_lp_amount: f64,
    ) -> (Self::Snapshot, Self::Snapshot) {
        let snapshot_before = Self::Snapshot::take(self);
        self.deposit(user, deposit, 0.0);
        let snapshot_after = Self::Snapshot::take(self);

        let title = format!(
            "Deposit {} a, {} b, expected lp: {expected_lp_amount}",
            deposit[0], deposit[1]
        );
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_deposit(
            snapshot_before.clone(),
            snapshot_after.clone(),
            user,
            deposit,
            expected_rewards,
            expected_lp_amount,
        );

        if expected_rewards.iter().sum::<f64>() != 0.0 {
            self.event_asserts()
                .assert_claimed_reward_event(user, expected_rewards);
        }

        (snapshot_before, snapshot_after)
    }

    fn do_swap<T: Into<usize> + Copy>(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        token_from: &Token<T>,
        token_to: &Token<T>,
        expected_receive_amount: f64,
        expected_fee: f64,
    ) -> (Self::Snapshot, Self::Snapshot) {
        let snapshot_before = Self::Snapshot::take(self);
        self.swap(
            sender,
            recipient,
            amount,
            receive_amount_min,
            token_from,
            token_to,
        );
        let snapshot_after = Self::Snapshot::take(self);

        let title = format!("Swap {amount} a => {expected_receive_amount} b");
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_swap(
            snapshot_before.clone(),
            snapshot_after.clone(),
            sender,
            recipient,
            token_from,
            token_to,
            amount,
            expected_receive_amount,
            expected_fee,
        );

        (snapshot_before, snapshot_after)
    }

    fn do_claim_admin_fee(&self, expected_rewards: [f64; N]) {
        let snapshot_before = Self::Snapshot::take(self);
        self.claim_admin_fee();
        let snapshot_after = Self::Snapshot::take(self);

        let title = format!("Claim admin fee, expected {:?}", expected_rewards);
        snapshot_before.print_change_with(&snapshot_after, &title);

        Self::assert_claim_admin_fee(&snapshot_before, &snapshot_after, expected_rewards);
    }
}
