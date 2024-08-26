use core::cmp::Ordering;

use shared::{require, soroban_data::SimpleSorobanData, utils::safe_cast, Error};
use soroban_sdk::{
    contracttype,
    token::{Client, TokenClient},
    Address, Env, Vec,
};

use crate::storage::{
    sized_array::{SizedAddressArray, SizedDecimalsArray, SizedU128Array},
    user_deposit::UserDeposit,
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

pub struct WithdrawAmount<const N: usize> {
    pub indexes: [usize; N],
    pub amounts: SizedU128Array,
    pub fees: SizedU128Array,
    pub new_token_balances: SizedU128Array,
}

pub struct DepositAmount {
    pub lp_amount: u128,
    pub new_token_balances: Vec<u128>,
}

#[contracttype]
#[derive(Debug)]
pub struct WithdrawAmountView {
    /// system precision
    pub amounts: Vec<u128>,
    /// token precision
    pub fees: Vec<u128>,
}

impl<const N: usize> From<WithdrawAmount<N>> for WithdrawAmountView {
    fn from(v: WithdrawAmount<N>) -> Self {
        Self {
            amounts: v.amounts.get_inner(),
            fees: v.fees.get_inner(),
        }
    }
}

pub trait PoolMath<const N: usize>: PoolStorage {
    fn get_current_d(&self) -> Result<u128, Error> {
        let mut values = [0; N];
        for (index, token_balance) in self.token_balances().iter().enumerate() {
            values[index] = token_balance;
        }

        self.get_d(values)
    }

    fn get_d(&self, values: [u128; N]) -> Result<u128, Error>;
    fn get_y(&self, values: [u128; N]) -> Result<u128, Error>;
}

pub trait Pool<const N: usize>: PoolStorage + SimpleSorobanData + PoolMath<N> {
    const BP: u128 = 10000;

    const MAX_A: u128 = 60;
    const MAX_TOKEN_BALANCE: u128 = 2u128.pow(40);
    const SYSTEM_PRECISION: u32 = 3;

    const P: u128 = 48;

    type Token: Into<usize> + Clone + Copy;

    /* Constructor  */

    fn from_init_params(
        env: &Env,
        a: u128,
        tokens: Vec<Address>,
        decimals: [u32; N],
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self;

    /* Methods */

    #[allow(clippy::too_many_arguments)]
    fn swap(
        &mut self,
        env: &Env,
        sender: Address,
        recipient: Address,
        amount: u128,
        receive_amount_min: u128,
        token_from: Self::Token,
        token_to: Self::Token,
    ) -> Result<(u128, u128), Error> {
        if amount == 0 {
            return Ok((0, 0));
        }

        let current_contract = env.current_contract_address();
        let receive_amount = self.get_receive_amount(amount, token_from, token_to)?;

        self.get_token(env, token_from)
            .transfer(&sender, &current_contract, &safe_cast(amount)?);

        self.token_balances_mut()
            .set(token_from, receive_amount.token_from_new_balance);
        self.token_balances_mut()
            .set(token_to, receive_amount.token_to_new_balance);

        self.add_rewards(receive_amount.fee, token_to.into());

        require!(
            receive_amount.output >= receive_amount_min,
            Error::InsufficientReceivedAmount
        );

        self.get_token(env, token_to).transfer(
            &current_contract,
            &recipient,
            &safe_cast(receive_amount.output)?,
        );

        Ok((receive_amount.output, receive_amount.fee))
    }

    fn deposit(
        &mut self,
        env: &Env,
        amounts: SizedU128Array,
        sender: Address,
        user_deposit: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<(SizedU128Array, u128), Error> {
        let current_contract = env.current_contract_address();

        if self.total_lp_amount() == 0 {
            let first = amounts.get(0usize);
            let is_deposit_valid = amounts.iter().all(|v| v == first);
            require!(is_deposit_valid, Error::InvalidFirstDeposit);
        }

        let deposit_amount = self.get_deposit_amount(env, amounts.clone())?;
        *self.token_balances_mut() = SizedU128Array::from_vec(deposit_amount.new_token_balances);

        require!(deposit_amount.lp_amount >= min_lp_amount, Error::Slippage);

        for (index, amount) in amounts.iter().enumerate() {
            if amount == 0 {
                continue;
            }

            self.get_token(env, index)
                .transfer(&sender, &current_contract, &safe_cast(amount)?);
        }

        let rewards = self.deposit_lp(env, user_deposit, deposit_amount.lp_amount)?;

        for (index, reward) in rewards.iter().enumerate() {
            if reward == 0 {
                continue;
            }

            self.get_token(env, index)
                .transfer(&current_contract, &sender, &safe_cast(reward)?);
        }

        Ok((rewards, deposit_amount.lp_amount))
    }

    fn withdraw(
        &mut self,
        env: &Env,
        sender: Address,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(WithdrawAmount<N>, SizedU128Array), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.total_lp_amount();
        let old_balances = self.token_balances().clone();
        let withdraw_amount = self.get_withdraw_amount(env, lp_amount)?;
        let rewards_amounts = self.withdraw_lp(env, user_deposit, lp_amount)?;

        for index in withdraw_amount.indexes {
            let token_amount =
                self.amount_from_system_precision(withdraw_amount.amounts.get(index), index);
            let token_amount = token_amount + rewards_amounts.get(index);

            self.add_rewards(withdraw_amount.fees.get(index), index);
            self.get_token(env, index).transfer(
                &current_contract,
                &sender,
                &safe_cast(token_amount)?,
            );
        }

        *self.token_balances_mut() = withdraw_amount.new_token_balances.clone();
        let d1 = self.total_lp_amount();

        let zero_balances_changes =
            (0..N).all(|index| self.token_balances().get(index) < old_balances.get(index));

        require!(zero_balances_changes && d1 < d0, Error::ZeroChanges);

        Ok((withdraw_amount, rewards_amounts))
    }

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

    fn add_rewards(&mut self, mut reward_amount: u128, token_index: usize) {
        if self.total_lp_amount() > 0 {
            let admin_fee_rewards = reward_amount * self.admin_fee_share_bp() / Self::BP;
            reward_amount -= admin_fee_rewards;

            let total_lp_amount = self.total_lp_amount();
            self.acc_rewards_per_share_p_mut()
                .add(token_index, (reward_amount << Self::P) / total_lp_amount);
            self.admin_fee_amount_mut()
                .add(token_index, admin_fee_rewards);
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

    /* Views */

    fn get_receive_amount(
        &self,
        input: u128,
        token_from: Self::Token,
        token_to: Self::Token,
    ) -> Result<ReceiveAmount, Error>;

    fn get_send_amount(
        &self,
        output: u128,
        token_from: Self::Token,
        token_to: Self::Token,
    ) -> Result<(u128, u128), Error>;

    fn get_withdraw_amount(&self, env: &Env, lp_amount: u128) -> Result<WithdrawAmount<N>, Error>;

    fn get_deposit_amount(
        &self,
        env: &Env,
        amounts: SizedU128Array,
    ) -> Result<DepositAmount, Error> {
        let d0 = self.total_lp_amount();

        let mut amounts_sp = [0; N];

        for (index, amounts_sp) in amounts_sp.iter_mut().enumerate() {
            *amounts_sp = self.amount_to_system_precision(amounts.get(index), index);
        }

        let amounts_sp = SizedU128Array::from_array(env, amounts_sp);

        let total_amount_sp = amounts_sp.iter().sum::<u128>();
        require!(total_amount_sp > 0, Error::ZeroAmount);

        let mut new_token_balances_sp = self.token_balances().clone();

        for (index, amount) in amounts.iter().enumerate() {
            if amount == 0 {
                continue;
            }

            new_token_balances_sp.add(index, amounts_sp.get(index));
        }

        let mut d_args = [0u128; N];

        for (index, d_arg) in d_args.iter_mut().enumerate() {
            *d_arg = new_token_balances_sp.get(index);
        }

        let d1 = self.get_d(d_args)?;

        require!(d1 > d0, Error::Forbidden);
        require!(
            new_token_balances_sp.iter().sum::<u128>() < Self::MAX_TOKEN_BALANCE,
            Error::PoolOverflow
        );

        let lp_amount = d1 - d0;

        Ok(DepositAmount {
            lp_amount,
            new_token_balances: new_token_balances_sp.get_inner(),
        })
    }

    /* Utils */

    fn amount_to_system_precision(&self, amount: u128, index: impl Into<usize>) -> u128 {
        let decimals = self.tokens_decimals().get(index);

        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            Ordering::Greater => amount / (10u128.pow(decimals - Self::SYSTEM_PRECISION)),
            Ordering::Less => amount * (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }

    fn amount_from_system_precision(&self, amount: u128, index: impl Into<usize>) -> u128 {
        let decimals = self.tokens_decimals().get(index);

        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            Ordering::Greater => amount * (10u128.pow(decimals - Self::SYSTEM_PRECISION)),
            Ordering::Less => amount / (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }
}
