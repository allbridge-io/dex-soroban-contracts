use core::cmp::Ordering;

use ethnum::U256;
use shared::{
    require,
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{Address, Env};

use crate::storage::{
    common::{Direction, Token},
    double_values::DoubleU128,
    pool::Pool,
    user_deposit::UserDeposit,
};

use super::pool_view::WithdrawAmount;

impl Pool {
    pub const BP: u128 = 10000;

    pub(crate) const MAX_TOKEN_BALANCE: u128 = 2u128.pow(40);
    pub(crate) const SYSTEM_PRECISION: u32 = 3;

    pub const P: u128 = 48;

    #[allow(clippy::too_many_arguments)]
    pub fn swap(
        &mut self,
        env: &Env,
        sender: Address,
        recipient: Address,
        amount: u128,
        receive_amount_min: u128,
        direction: Direction,
    ) -> Result<(u128, u128), Error> {
        if amount == 0 {
            return Ok((0, 0));
        }

        let current_contract = env.current_contract_address();
        let (token_from, token_to) = direction.get_tokens();
        let receive_amount = self.get_receive_amount(amount, token_from)?;

        self.get_token(env, token_from)
            .transfer(&sender, &current_contract, &safe_cast(amount)?);

        self.token_balances[token_from] = receive_amount.token_from_new_balance;
        self.token_balances[token_to] = receive_amount.token_to_new_balance;

        self.add_rewards(receive_amount.fee, token_to);

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

    pub fn deposit(
        &mut self,
        env: &Env,
        amounts: DoubleU128,
        sender: Address,
        user_deposit: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<(DoubleU128, u128), Error> {
        let current_contract = env.current_contract_address();

        if self.total_lp_amount == 0 {
            require!(amounts.data.0 == amounts.data.1, Error::InvalidFirstDeposit);
        }

        let deposit_amount = self.get_deposit_amount(amounts.clone())?;
        self.token_balances = deposit_amount.new_token_balances;

        require!(deposit_amount.lp_amount >= min_lp_amount, Error::Slippage);

        for (index, amount) in amounts.to_array().into_iter().enumerate() {
            if amount == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &sender,
                &current_contract,
                &safe_cast(amount)?,
            );
        }

        let rewards = self.deposit_lp(user_deposit, deposit_amount.lp_amount)?;

        for (index, reward) in rewards.to_array().into_iter().enumerate() {
            if reward == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &safe_cast(reward)?,
            );
        }

        Ok((rewards, deposit_amount.lp_amount))
    }

    pub fn withdraw(
        &mut self,
        env: &Env,
        sender: Address,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(WithdrawAmount, DoubleU128), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.total_lp_amount;
        let old_balances = self.token_balances.clone();
        let withdraw_amount = self.get_withdraw_amount(lp_amount)?;
        let rewards_amounts = self.withdraw_lp(user_deposit, lp_amount)?;

        for index in withdraw_amount.indexes {
            let token_amount = self.amount_from_system_precision(
                withdraw_amount.amounts[index],
                self.tokens_decimals[index],
            );
            let token_amount = token_amount + rewards_amounts[index];

            self.add_rewards(withdraw_amount.fees[index], index.into());
            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &safe_cast(token_amount)?,
            );
        }

        self.token_balances = withdraw_amount.new_token_balances.clone();
        let d1 = self.total_lp_amount;

        require!(
            self.token_balances[0] < old_balances[0]
                && self.token_balances[1] < old_balances[1]
                && d1 < d0,
            Error::ZeroChanges
        );

        Ok((withdraw_amount, rewards_amounts))
    }

    pub(crate) fn deposit_lp(
        &mut self,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<DoubleU128, Error> {
        let pending = self.get_pending(user_deposit);

        self.total_lp_amount += lp_amount;
        user_deposit.lp_amount += lp_amount;
        user_deposit.reward_debts = self.get_reward_debts(user_deposit);

        Ok(pending)
    }

    pub(crate) fn withdraw_lp(
        &mut self,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<DoubleU128, Error> {
        require!(user_deposit.lp_amount >= lp_amount, Error::NotEnoughAmount);

        let pending = self.get_pending(user_deposit);

        self.total_lp_amount -= lp_amount;
        user_deposit.lp_amount -= lp_amount;
        user_deposit.reward_debts = self.get_reward_debts(user_deposit);

        Ok(pending)
    }

    pub fn claim_rewards(
        &self,
        env: &Env,
        user: Address,
        user_deposit: &mut UserDeposit,
    ) -> Result<DoubleU128, Error> {
        let mut pending = DoubleU128::default();

        if user_deposit.lp_amount == 0 {
            return Ok(pending);
        }

        let rewards = self.get_reward_debts(user_deposit);

        for (index, reward) in rewards.to_array().into_iter().enumerate() {
            pending[index] = reward - user_deposit.reward_debts[index];

            if pending[index] > 0 {
                user_deposit.reward_debts[index] = reward;

                self.get_token_by_index(env, index).transfer(
                    &env.current_contract_address(),
                    &user,
                    &safe_cast(pending[index])?,
                );
            }
        }

        Ok(pending)
    }

    pub(crate) fn add_rewards(&mut self, mut reward_amount: u128, token: Token) {
        if self.total_lp_amount > 0 {
            let admin_fee_rewards = reward_amount * self.admin_fee_share_bp / Pool::BP;
            reward_amount -= admin_fee_rewards;
            self.acc_rewards_per_share_p[token] +=
                (reward_amount << Pool::P) / self.total_lp_amount;
            self.admin_fee_amount[token] += admin_fee_rewards;
        }
    }

    pub fn get_pending(&self, user_deposit: &UserDeposit) -> DoubleU128 {
        if user_deposit.lp_amount == 0 {
            return DoubleU128::default();
        }

        DoubleU128::from((
            ((user_deposit.lp_amount * self.acc_rewards_per_share_p[0]) >> Pool::P)
                - user_deposit.reward_debts[0],
            ((user_deposit.lp_amount * self.acc_rewards_per_share_p[1]) >> Pool::P)
                - user_deposit.reward_debts[1],
        ))
    }

    pub fn get_reward_debts(&self, user_deposit: &UserDeposit) -> DoubleU128 {
        DoubleU128::from((
            (user_deposit.lp_amount * self.acc_rewards_per_share_p[0]) >> Pool::P,
            (user_deposit.lp_amount * self.acc_rewards_per_share_p[1]) >> Pool::P,
        ))
    }

    // y = (sqrt(x(4AD³ + x (4A(D - x) - D )²)) + x (4A(D - x) - D ))/8Ax
    pub fn get_y(&self, native_x: u128, d: u128) -> Result<u128, Error> {
        let a4 = self.a << 2;

        let int_a4: i128 = safe_cast(a4)?;
        let int_d: i128 = safe_cast(d)?;
        let int_native_x: i128 = safe_cast(native_x)?;

        let ddd = U256::new(d * d) * d;
        // 4A(D - x) - D
        let part1 = int_a4 * (int_d - int_native_x) - int_d;
        // x * (4AD³ + x(part1²))
        let part2 = (ddd * a4 + (U256::new(safe_cast(part1 * part1)?) * native_x)) * native_x;
        // (sqrt(part2) + x(part1))
        let sqrt_sum = safe_cast::<u128, i128>(sqrt(&part2).as_u128())? + (int_native_x * part1);
        // (sqrt(part2) + x(part1)) / 8Ax)
        Ok(safe_cast::<i128, u128>(sqrt_sum)? / ((self.a << 3) * native_x))
    }

    pub fn get_current_d(&self) -> u128 {
        self.get_d(self.token_balances[0], self.token_balances[1])
    }

    pub fn get_d(&self, x: u128, y: u128) -> u128 {
        let xy: u128 = x * y;
        // Axy(x+y)
        let p1 = U256::new(self.a * (x + y) * xy);

        // xy(4A - 1) / 3
        let p2 = U256::new(xy * ((self.a << 2) - 1) / 3);

        // sqrt(p1² + p2³)
        let p3 = sqrt(&((p1 * p1) + (p2 * p2 * p2)));

        // cbrt(p1 + p3) + cbrt(p1 - p3)
        let mut d = cbrt(&(p1 + p3));
        if p3.gt(&p1) {
            d -= cbrt(&(p3 - p1));
        } else {
            d += cbrt(&(p1 - p3));
        }
        d << 1
    }

    pub(crate) fn amount_to_system_precision(&self, amount: u128, decimals: u32) -> u128 {
        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            Ordering::Greater => amount / (10u128.pow(decimals - Self::SYSTEM_PRECISION)),
            Ordering::Less => amount * (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }

    pub(crate) fn amount_from_system_precision(&self, amount: u128, decimals: u32) -> u128 {
        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            Ordering::Greater => amount * (10u128.pow(decimals - Self::SYSTEM_PRECISION)),
            Ordering::Less => amount / (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            Ordering::Equal => amount,
        }
    }
}
