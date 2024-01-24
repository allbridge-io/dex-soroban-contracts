use ethnum::U256;
use shared::{require, utils::num::*, Error};
use soroban_sdk::{contracttype, Address, Env};

use crate::storage::{
    double_u128::DoubleU128,
    pool::{Pool, Token},
    user_deposit::UserDeposit,
};

#[contracttype]
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    A2B,
    B2A,
}

impl Direction {
    #[inline]
    pub fn get_tokens(&self) -> (Token, Token) {
        match self {
            Direction::A2B => (Token::A, Token::B),
            Direction::B2A => (Token::B, Token::A),
        }
    }
}

impl Pool {
    const BP: u128 = 10000;

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
        let current_contract = env.current_contract_address();
        let (token_from, token_to) = direction.get_tokens();
        let d0 = self.get_current_d();

        if amount == 0 {
            return Ok((0, 0));
        }

        self.get_token(env, token_from)
            .transfer(&sender, &current_contract, &(amount as i128));

        let mut result = 0;

        self.token_balances[token_from] += amount;

        let token_to_new_amount = self.get_y(self.token_balances[token_from], d0);
        if self.token_balances[token_to] > token_to_new_amount {
            result = self.token_balances[token_to] - token_to_new_amount;
        }

        let fee = result * self.fee_share_bp / Self::BP;

        result -= fee;

        self.token_balances[token_to] = token_to_new_amount;

        self.add_rewards(fee, token_to);

        require!(
            result >= receive_amount_min,
            Error::InsufficientReceivedAmount
        );

        self.get_token(env, token_to)
            .transfer(&current_contract, &recipient, &(result as i128));

        Ok((result, fee))
    }

    pub fn deposit(
        &mut self,
        env: &Env,
        amounts: DoubleU128,
        sender: Address,
        user: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<(DoubleU128, u128), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.get_current_d();

        let total_amount: u128 = amounts.to_array().iter().sum();
        require!(total_amount > 0, Error::ZeroAmount);

        for (index, amount) in amounts.to_array().into_iter().enumerate() {
            if amount == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &sender,
                &current_contract,
                &(amounts[index] as i128),
            );

            self.token_balances[index] += amount;
        }

        let d1 = self.get_current_d();

        require!(d1 > d0, Error::Forbidden);

        let lp_amount = if self.total_lp_amount == 0 {
            d1
        } else {
            self.total_lp_amount * (d1 - d0) / d0
        };

        require!(lp_amount >= min_lp_amount, Error::Slippage);

        let rewards = self.deposit_lp(user, lp_amount)?;

        for (index, reward) in rewards.to_array().into_iter().enumerate() {
            if reward == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &(reward as i128),
            );
        }

        Ok((rewards, lp_amount))
    }

    pub fn withdraw(
        &mut self,
        env: &Env,
        sender: Address,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(DoubleU128, DoubleU128), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.get_current_d();
        let old_balances: u128 = self.token_balances.to_array().iter().sum();
        let old_total_lp_amount = self.total_lp_amount;
        let rewards_amounts = self.withdraw_lp(user, lp_amount)?;
        let mut amounts = DoubleU128::default();

        for (index, token_balance) in self.token_balances.to_array().into_iter().enumerate() {
            let token_amount = token_balance * lp_amount / old_total_lp_amount;
            amounts[index] = token_amount;
            let withdraw_amount = token_amount + rewards_amounts[index];

            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &(withdraw_amount as i128),
            );
            self.token_balances[index] -= withdraw_amount;
        }

        let new_balances: u128 = self.token_balances.to_array().iter().sum();
        let d1 = self.get_current_d();

        require!(new_balances < old_balances && d1 < d0, Error::ZeroChanges);

        Ok((amounts, rewards_amounts))
    }

    pub(crate) fn deposit_lp(
        &mut self,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<DoubleU128, Error> {
        let pending = self.get_pending(user);

        self.total_lp_amount += lp_amount;
        user.lp_amount += lp_amount;
        user.reward_debts = self.get_reward_depts(user);

        Ok(pending)
    }

    pub(crate) fn withdraw_lp(
        &mut self,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<DoubleU128, Error> {
        require!(user.lp_amount >= lp_amount, Error::NotEnoughAmount);

        let pending = self.get_pending(user);

        self.total_lp_amount -= lp_amount;
        user.lp_amount -= lp_amount;
        user.reward_debts = self.get_reward_depts(user);

        Ok(pending)
    }

    pub fn claim_rewards(
        &self,
        env: &Env,
        sender: Address,
        user: &mut UserDeposit,
    ) -> Result<DoubleU128, Error> {
        let mut pending = DoubleU128::default();

        if user.lp_amount == 0 {
            return Ok(pending);
        }

        let rewads = self.get_reward_depts(user);

        for (index, reward) in rewads.to_array().into_iter().enumerate() {
            pending[index] = reward - user.reward_debts[index];

            if pending[index] > 0 {
                user.reward_debts[index] = reward;

                self.get_token_by_index(env, index).transfer(
                    &env.current_contract_address(),
                    &sender,
                    &(pending[index] as i128),
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

    pub fn get_pending(&self, user: &UserDeposit) -> DoubleU128 {
        if user.lp_amount == 0 {
            return DoubleU128::default();
        }

        DoubleU128::from((
            ((user.lp_amount * self.acc_rewards_per_share_p[0]) >> Pool::P) - user.reward_debts[0],
            ((user.lp_amount * self.acc_rewards_per_share_p[1]) >> Pool::P) - user.reward_debts[1],
        ))
    }

    pub fn get_reward_depts(&self, user: &UserDeposit) -> DoubleU128 {
        DoubleU128::from((
            (user.lp_amount * self.acc_rewards_per_share_p[0]) >> Pool::P,
            (user.lp_amount * self.acc_rewards_per_share_p[1]) >> Pool::P,
        ))
    }

    // y = (sqrt(x(4AD³ + x (4A(D - x) - D )²)) + x (4A(D - x) - D ))/8Ax
    pub fn get_y(&self, native_x: u128, d: u128) -> u128 {
        let a4 = self.a << 2;
        let ddd = U256::new(d * d) * d;
        // 4A(D - x) - D
        let part1 = a4 as i128 * (d as i128 - native_x as i128) - d as i128;
        // x * (4AD³ + x(part1²))
        let part2 = (ddd * a4 + (U256::new((part1 * part1) as u128) * native_x)) * native_x;
        // (sqrt(part2) + x(part1)) / 8Ax)
        (sqrt(&part2).as_u128() as i128 + (native_x as i128 * part1)) as u128
            / ((self.a << 3) * native_x)
    }

    pub fn get_current_d(&self) -> u128 {
        self.get_d(self.token_balances[0] >> 14, self.token_balances[1] >> 14) << 14
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
}
