use core::cmp::Ordering;

use shared::{require, soroban_data::SimpleSorobanData, utils::safe_cast, Error};
use soroban_sdk::{
    token::{Client, TokenClient},
    Address, Env, Vec,
};

use crate::{
    events::{DepositEvent, RewardsClaimedEvent, WithdrawEvent},
    storage::{
        common::Token,
        sized_array::{SizedAddressArray, SizedDecimalsArray, SizedU128Array},
        user_deposit::UserDeposit,
    },
};

pub trait PoolStorage {
    fn a(&self) -> u128;
    fn fee_share_bp(&self) -> u128;
    fn admin_fee_share_bp(&self) -> u128;
    fn total_lp_amount(&self) -> u128;
    fn tokens(&self) -> &SizedAddressArray;
    fn tokens_decimals(&self) -> &SizedDecimalsArray;
    fn token_balances(&self) -> &SizedU128Array;
    fn acc_rewards_per_share_p(&self) -> &SizedU128Array;
    fn admin_fee_amount(&self) -> &SizedU128Array;

    fn a_mut(&mut self) -> &mut u128;
    fn fee_share_bp_mut(&mut self) -> &mut u128;
    fn admin_fee_share_bp_mut(&mut self) -> &mut u128;
    fn total_lp_amount_mut(&mut self) -> &mut u128;
    fn tokens_mut(&mut self) -> &mut SizedAddressArray;
    fn tokens_decimals_mut(&mut self) -> &mut SizedDecimalsArray;
    fn token_balances_mut(&mut self) -> &mut SizedU128Array;
    fn acc_rewards_per_share_p_mut(&mut self) -> &mut SizedU128Array;
    fn admin_fee_amount_mut(&mut self) -> &mut SizedU128Array;

    #[inline]
    fn get_token(&self, env: &Env, index: impl Into<usize>) -> TokenClient<'_> {
        Client::new(env, &self.tokens().get(index.into()))
    }
}

pub struct ReceiveAmount {
    pub token_from_new_balance: u128,
    pub token_to_new_balance: u128,
    pub output: u128,
    pub fee: u128,
}

pub struct WithdrawAmount {
    pub indexes: [usize; 3],
    pub amounts: SizedU128Array,
    pub fees: SizedU128Array,
    pub new_token_balances: SizedU128Array,
}

pub struct DepositAmount {
    pub lp_amount: u128,
    pub new_token_balances: Vec<u128>,
}

pub trait PoolView {
    fn get_receive_amount(
        &self,
        input: u128,
        token_from: Token,
        token_to: Token,
    ) -> Result<ReceiveAmount, Error>;

    fn get_send_amount(
        &self,
        output: u128,
        token_from: Token,
        token_to: Token,
    ) -> Result<(u128, u128), Error>;

    fn get_withdraw_amount(&self, env: &Env, lp_amount: u128) -> Result<WithdrawAmount, Error>;

    fn get_deposit_amount(
        &self,
        env: &Env,
        amounts: SizedU128Array,
    ) -> Result<DepositAmount, Error>;
}

pub trait Pool<const N: usize>: PoolStorage + PoolView + SimpleSorobanData {
    const BP: u128 = 10000;

    const MAX_A: u128 = 60;
    const MAX_TOKEN_BALANCE: u128 = 2u128.pow(40);
    const SYSTEM_PRECISION: u32 = 3;

    const P: u128 = 48;

    type Deposit: DepositEvent;
    type RewardsClaimed: RewardsClaimedEvent;
    type Withdraw: WithdrawEvent;

    /* Contructor  */

    fn from_init_params(
        env: &Env,
        a: u128,
        tokens: [Address; N],
        decimals: [u32; N],
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self;

    /* Methods */

    fn swap(
        &mut self,
        env: &Env,
        sender: Address,
        recipient: Address,
        amount: u128,
        receive_amount_min: u128,
        token_from: Token,
        token_to: Token,
    ) -> Result<(u128, u128), Error>;

    fn deposit(
        &mut self,
        env: &Env,
        amounts: SizedU128Array,
        sender: Address,
        user_deposit: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<(SizedU128Array, u128), Error>;

    fn withdraw(
        &mut self,
        env: &Env,
        sender: Address,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(WithdrawAmount, SizedU128Array), Error>;

    fn get_current_d(&self) -> Result<u128, Error>;
    fn get_d(&self, values: [u128; N]) -> Result<u128, Error>;
    fn get_y(&self, values: [u128; N]) -> Result<u128, Error>;

    fn deposit_lp(
        &mut self,
        env: &Env,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<SizedU128Array, Error> {
        let pending = self.get_pending(env, user_deposit);

        *self.total_lp_amount_mut() += lp_amount;
        user_deposit.lp_amount += lp_amount;
        user_deposit.reward_debts = self.get_reward_debts(env, user_deposit);

        Ok(pending)
    }

    fn withdraw_lp(
        &mut self,
        env: &Env,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<SizedU128Array, Error> {
        require!(user_deposit.lp_amount >= lp_amount, Error::NotEnoughAmount);

        let pending = self.get_pending(env, user_deposit);

        *self.total_lp_amount_mut() -= lp_amount;
        user_deposit.lp_amount -= lp_amount;
        user_deposit.reward_debts = self.get_reward_debts(env, user_deposit);

        Ok(pending)
    }

    fn claim_rewards(
        &self,
        env: &Env,
        user: Address,
        user_deposit: &mut UserDeposit,
    ) -> Result<SizedU128Array, Error> {
        let mut pending = SizedU128Array::default_val::<N>(env);

        if user_deposit.lp_amount == 0 {
            return Ok(pending);
        }

        let rewards = self.get_reward_debts(env, user_deposit);

        for (index, reward) in rewards.iter().enumerate() {
            pending.set(index, reward - user_deposit.reward_debts.get(index));

            if pending.get(index) > 0 {
                user_deposit.reward_debts.set(index, reward);

                self.get_token(env, index).transfer(
                    &env.current_contract_address(),
                    &user,
                    &safe_cast(pending.get(index))?,
                );
            }
        }

        Ok(pending)
    }

    fn add_rewards(&mut self, mut reward_amount: u128, token: Token) {
        if self.total_lp_amount() > 0 {
            let admin_fee_rewards = reward_amount * self.admin_fee_share_bp() / Self::BP;
            reward_amount -= admin_fee_rewards;

            let total_lp_amount = self.total_lp_amount();
            self.acc_rewards_per_share_p_mut()
                .add(token, (reward_amount << Self::P) / total_lp_amount);
            self.admin_fee_amount_mut().add(token, admin_fee_rewards);
        }
    }

    fn get_pending(&self, env: &Env, user_deposit: &UserDeposit) -> SizedU128Array {
        if user_deposit.lp_amount == 0 {
            return SizedU128Array::default_val::<N>(env);
        }

        let rewards = self.get_reward_debts(env, user_deposit);

        rewards - user_deposit.reward_debts.clone()
    }

    fn get_reward_debts(&self, env: &Env, user_deposit: &UserDeposit) -> SizedU128Array {
        let mut v = SizedU128Array::default_val::<N>(env);

        for (index, acc_rewards_per_share_p) in self.acc_rewards_per_share_p().iter().enumerate() {
            let new_acc_rewards = (user_deposit.lp_amount * acc_rewards_per_share_p) >> Self::P;
            v.set(index, new_acc_rewards);
        }

        v
    }

    fn amount_to_system_precision(&self, amount: u128, decimals: u32) -> u128 {
        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            Ordering::Greater => amount / (10u128.pow(decimals - Self::SYSTEM_PRECISION)),
            Ordering::Less => amount * (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }

    fn amount_from_system_precision(&self, amount: u128, decimals: u32) -> u128 {
        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            Ordering::Greater => amount * (10u128.pow(decimals - Self::SYSTEM_PRECISION)),
            Ordering::Less => amount / (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }
}
