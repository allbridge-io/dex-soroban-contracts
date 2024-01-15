use ethnum::U256;
use shared::{require, utils::num::*, Error};
use soroban_sdk::{contracttype, Address, Env};

use crate::storage::{
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
        zero_fee: bool,
        direction: Direction,
    ) -> Result<(u128, u128), Error> {
        let current_contract = env.current_contract_address();
        let (token_from, token_to) = direction.get_tokens();
        self.get_token(env, token_from)
            .transfer(&sender, &current_contract, &(amount as i128));

        let mut result = 0;

        if amount == 0 {
            return Ok((0, 0));
        }

        self.set_token_balance(self.get_token_balance(token_from) + amount, token_from);

        let token_to_new_amount = self.get_y(self.get_token_balance(token_from));
        if self.get_token_balance(token_from) > token_to_new_amount {
            result = self.get_token_balance(token_to) - token_to_new_amount;
        }

        let fee = if zero_fee {
            0
        } else {
            result * self.fee_share_bp / Self::BP
        };

        result -= fee;

        self.set_token_balance(token_to_new_amount, token_to);
        self.add_rewards(fee, Token::A);

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
        amounts: (u128, u128),
        sender: Address,
        user: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<((u128, u128), u128), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.d;

        require!(amounts.0 > 0 && amounts.1 > 0, Error::ZeroAmount);

        self.get_token(env, Token::A)
            .transfer(&sender, &current_contract, &(amounts.0 as i128));
        self.get_token(env, Token::B)
            .transfer(&sender, &current_contract, &(amounts.1 as i128));

        self.token_a_balance += amounts.0;
        self.token_b_balance += amounts.1;

        self.update_d();

        require!(self.d > d0, Error::Forbidden);

        let lp_amount = if self.total_lp_amount == 0 {
            self.d
        } else {
            self.total_lp_amount * (self.d - d0) / d0
        };

        require!(lp_amount >= min_lp_amount, Error::Slippage);

        let rewards = self.deposit_lp(user, lp_amount)?;

        self.get_token(&env, Token::A)
            .transfer(&current_contract, &sender, &(rewards.0 as i128));
        self.get_token(&env, Token::B)
            .transfer(&current_contract, &sender, &(rewards.1 as i128));

        Ok((rewards, lp_amount))
    }

    pub fn withdraw(
        &mut self,
        env: &Env,
        sender: Address,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(), Error> {
        let old_balance = self.token_a_balance + self.token_b_balance;
        let current_contract = env.current_contract_address();
        let d0 = self.d;
        let token_a_amount = self.token_a_balance * lp_amount / self.total_lp_amount;
        let token_b_amount = self.token_b_balance * lp_amount / self.total_lp_amount;

        self.token_a_balance -= token_a_amount;
        self.token_b_balance -= token_b_amount;

        let rewards_amounts = self.withdraw_lp(user, lp_amount)?;

        require!(
            self.token_a_balance + self.token_b_balance < old_balance,
            Error::ZeroChanges
        );

        let d1 = self.get_d(self.token_a_balance, self.token_b_balance);

        require!(d1 < d0, Error::ZeroChanges);

        self.get_token(&env, Token::A).transfer(
            &current_contract,
            &sender,
            &((token_a_amount + rewards_amounts.0) as i128),
        );
        self.get_token(&env, Token::B).transfer(
            &current_contract,
            &sender,
            &((token_b_amount + rewards_amounts.1) as i128),
        );

        Ok(())
    }

    pub(crate) fn deposit_lp(
        &mut self,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(u128, u128), Error> {
        let mut pending = (0, 0);
        if user.lp_amount > 0 {
            pending = (
                ((user.lp_amount * self.acc_reward_a_per_share_p) >> Pool::P) - user.reward_debts.0,
                ((user.lp_amount * self.acc_reward_b_per_share_p) >> Pool::P) - user.reward_debts.1,
            )
        };

        self.total_lp_amount += lp_amount;
        user.lp_amount += lp_amount;

        user.reward_debts = (
            (user.lp_amount * self.acc_reward_a_per_share_p) >> Pool::P,
            (user.lp_amount * self.acc_reward_b_per_share_p) >> Pool::P,
        );

        Ok(pending)
    }

    pub(crate) fn withdraw_lp(
        &mut self,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(u128, u128), Error> {
        require!(user.lp_amount >= lp_amount, Error::NotEnoughAmount);

        let mut pending = (0, 0);
        if user.lp_amount > 0 {
            pending = (
                ((user.lp_amount * self.acc_reward_a_per_share_p) >> Pool::P) - user.reward_debts.0,
                ((user.lp_amount * self.acc_reward_b_per_share_p) >> Pool::P) - user.reward_debts.1,
            )
        }

        self.total_lp_amount -= lp_amount;
        user.lp_amount -= lp_amount;

        user.reward_debts = (
            (user.lp_amount * self.acc_reward_a_per_share_p) >> Pool::P,
            (user.lp_amount * self.acc_reward_b_per_share_p) >> Pool::P,
        );

        Ok(pending)
    }

    pub fn claim_rewards(&self, user_deposit: &mut UserDeposit) -> Result<(u128, u128), Error> {
        let mut pending = (0, 0);

        if user_deposit.lp_amount > 0 {
            let rewards_a = (user_deposit.lp_amount * self.acc_reward_a_per_share_p) >> Pool::P;
            let rewards_b = (user_deposit.lp_amount * self.acc_reward_b_per_share_p) >> Pool::P;

            pending.0 = rewards_a - user_deposit.reward_debts.0;
            if pending.0 > 0 {
                user_deposit.reward_debts.0 = rewards_a;
            }

            pending.1 = rewards_b - user_deposit.reward_debts.1;
            if pending.1 > 0 {
                user_deposit.reward_debts.1 = rewards_b;
            }

            return Ok((pending.0, pending.1));
        }

        Ok(pending)
    }

    pub(crate) fn add_rewards(&mut self, mut reward_amount: u128, token: Token) {
        if self.total_lp_amount > 0 {
            let admin_fee_rewards = reward_amount * self.admin_fee_share_bp / Pool::BP;
            reward_amount -= admin_fee_rewards;
            let share = (reward_amount << Pool::P) / self.total_lp_amount;

            match token {
                Token::A => self.acc_reward_a_per_share_p += share,
                Token::B => self.acc_reward_b_per_share_p += share,
            }

            self.admin_fee_amount += admin_fee_rewards;
        }
    }

    // y = (sqrt(x(4AD³ + x (4A(D - x) - D )²)) + x (4A(D - x) - D ))/8Ax
    pub fn get_y(&self, native_x: u128) -> u128 {
        let a4 = self.a << 2;
        let ddd = U256::new(self.d * self.d) * self.d;
        // 4A(D - x) - D
        let part1 = a4 as i128 * (self.d as i128 - native_x as i128) - self.d as i128;
        // x * (4AD³ + x(part1²))
        let part2 = (ddd * a4 + (U256::new((part1 * part1) as u128) * native_x)) * native_x;
        // (sqrt(part2) + x(part1)) / 8Ax)
        (sqrt(&part2).as_u128() as i128 + (native_x as i128 * part1)) as u128
            / ((self.a << 3) * native_x)
    }

    fn update_d(&mut self) {
        self.d = self.get_d(self.token_a_balance, self.token_b_balance);
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
