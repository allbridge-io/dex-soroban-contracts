use soroban_sdk::Env;

use crate::{
    contracts::three_pool::{Deposit, RewardsClaimed, Swapped, Withdraw},
    utils::{assert_rel_eq, float_to_uint, float_to_uint_sp, get_latest_event},
};

use super::{Token, User};

pub struct EventAsserts<const N: usize>(pub Env);

impl<const N: usize> EventAsserts<N> {
    pub fn assert_withdraw_event(
        &self,
        expected_user: &User,
        lp_amount: f64,
        amounts: [f64; N],
        fees: [f64; N],
    ) {
        let withdraw = get_latest_event::<Withdraw>(&self.0).expect("Expected Withdraw");

        assert_eq!(withdraw.user, expected_user.as_address());
        assert_eq!(withdraw.lp_amount, float_to_uint_sp(lp_amount));

        for (index, amount) in amounts.iter().enumerate() {
            assert_rel_eq(
                withdraw.amounts.get_unchecked(index as u32),
                float_to_uint_sp(*amount),
                1,
            );
        }

        for (index, fee) in fees.iter().enumerate() {
            assert_rel_eq(
                withdraw.fees.get_unchecked(index as u32),
                float_to_uint(*fee, 7),
                1,
            );
        }
    }

    pub fn assert_deposit_event(
        &self,
        expected_user: &User,
        expected_lp_amount: f64,
        tokens: [f64; N],
    ) {
        let deposit = get_latest_event::<Deposit>(&self.0).expect("Expected Deposit");

        for (index, token) in tokens.iter().enumerate() {
            assert_eq!(
                deposit.amounts.get_unchecked(index as u32),
                float_to_uint(*token, 7)
            );
        }

        assert_eq!(deposit.user, expected_user.as_address());
        assert_eq!(float_to_uint_sp(expected_lp_amount), deposit.lp_amount);
    }

    pub fn assert_claimed_reward_event(&self, expected_user: &User, expected_rewards: [f64; N]) {
        let rewards_claimed =
            get_latest_event::<RewardsClaimed>(&self.0).expect("Expected RewardsClaimed");

        assert_eq!(rewards_claimed.user, expected_user.as_address());

        for (i, reward) in rewards_claimed.rewards.iter().enumerate() {
            assert_rel_eq(reward, float_to_uint(expected_rewards[i], 7), 1);
        }
    }

    pub fn assert_swapped_event(
        &self,
        sender: &User,
        recipient: &User,
        token_from: &Token<impl Into<usize>>,
        token_to: &Token<impl Into<usize>>,
        from_amount: f64,
        expected_to_amount: f64,
        expected_fee: f64,
    ) {
        let swapped = get_latest_event::<Swapped>(&self.0).expect("Expected Swapped");

        assert_eq!(swapped.sender, sender.as_address());
        assert_eq!(swapped.recipient, recipient.as_address());

        assert_eq!(swapped.from_amount, float_to_uint(from_amount, 7));
        assert_eq!(swapped.to_amount, float_to_uint(expected_to_amount, 7));
        assert_rel_eq(swapped.fee, float_to_uint(expected_fee, 7), 1);

        assert_eq!(swapped.from_token, token_from.id);
        assert_eq!(swapped.to_token, token_to.id);
    }
}
