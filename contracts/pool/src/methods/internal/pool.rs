use ethnum::U256;
use shared::{require, utils::num::*, Error};
use soroban_sdk::{contracttype, vec, Address, Env, Vec};

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
        self.get_token_by_index(env, token_from as usize).transfer(
            &sender,
            &current_contract,
            &(amount as i128),
        );

        let mut result = 0;

        if amount == 0 {
            return Ok((0, 0));
        }

        let token_from_balance = self.token_balances.get_unchecked(token_from as u32);
        self.token_balances
            .set(token_from as u32, token_from_balance + amount);

        let token_from_balance = self.token_balances.get_unchecked(token_from as u32);

        let token_to_new_amount = self.get_y(token_from_balance);
        if token_from_balance > token_to_new_amount {
            result = self.token_balances.get_unchecked(token_to as u32) - token_to_new_amount;
        }

        let fee = if zero_fee {
            0
        } else {
            result * self.fee_share_bp / Self::BP
        };

        result -= fee;

        self.token_balances
            .set(token_to as u32, token_to_new_amount);

        self.add_rewards(fee, token_to);

        require!(
            result >= receive_amount_min,
            Error::InsufficientReceivedAmount
        );

        self.get_token_by_index(env, token_to as usize).transfer(
            &current_contract,
            &recipient,
            &(result as i128),
        );

        Ok((result, fee))
    }

    pub fn deposit(
        &mut self,
        env: &Env,
        amounts: (u128, u128),
        sender: Address,
        user: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<([u128; 2], u128), Error> {
        let amounts = vec![env, amounts.0, amounts.1];

        let current_contract = env.current_contract_address();
        let d0 = self.get_current_d();

        let total_amount: u128 = amounts.iter().sum();
        require!(total_amount > 0, Error::ZeroAmount);

        for (index, amount) in amounts.iter().enumerate() {
            if amount == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &sender,
                &current_contract,
                &(amounts.get_unchecked(index as u32) as i128),
            );
            let token_balance = self.token_balances.get_unchecked(index as u32);

            self.token_balances
                .set(index as u32, token_balance + amount);
        }

        let d1 = self.get_current_d();

        require!(d1 > d0, Error::Forbidden);

        let lp_amount = if self.total_lp_amount == 0 {
            d1
        } else {
            self.total_lp_amount * (d1 - d0) / d0
        };

        require!(lp_amount >= min_lp_amount, Error::Slippage);

        let rewards = self.deposit_lp(env, user, lp_amount)?;

        for (index, reward) in rewards.iter().enumerate() {
            if *reward == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &(*reward as i128),
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
    ) -> Result<(), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.get_current_d();

        let old_balances: u128 = self.token_balances.iter().sum();
        let rewards_amounts = self.withdraw_lp(env, user, lp_amount)?;

        for (index, token_balance) in self.token_balances.iter().enumerate() {
            let token_amount = token_balance * lp_amount / self.total_lp_amount;
            let token_balance = token_balance - token_amount;
            let transfer_amount = token_amount + rewards_amounts[index];

            self.token_balances.set(index as u32, token_balance);
            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &(transfer_amount as i128),
            );
        }

        let new_balances: u128 = self.token_balances.iter().sum();
        let d1 = self.get_current_d();

        require!(new_balances < old_balances && d1 < d0, Error::ZeroChanges);

        Ok(())
    }

    pub(crate) fn deposit_lp(
        &mut self,
        env: &Env,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<[u128; 2], Error> {
        let mut pending = [0, 0];

        if user.lp_amount > 0 {
            pending = self.get_pending(user);
        };

        self.total_lp_amount += lp_amount;
        user.lp_amount += lp_amount;

        user.reward_debts = Vec::from_array(env, self.get_reward_depts(user));

        Ok(pending)
    }

    pub(crate) fn withdraw_lp(
        &mut self,
        env: &Env,
        user: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<[u128; 2], Error> {
        require!(user.lp_amount >= lp_amount, Error::NotEnoughAmount);

        let mut pending = [0, 0];
        if user.lp_amount > 0 {
            pending = self.get_pending(user);
        }

        self.total_lp_amount -= lp_amount;
        user.lp_amount -= lp_amount;

        user.reward_debts = Vec::from_array(env, self.get_reward_depts(user));

        Ok(pending)
    }

    pub fn claim_rewards(&self, user: &mut UserDeposit) -> Result<[u128; 2], Error> {
        let mut pending = [0, 0];

        if user.lp_amount > 0 {
            let rewads = self.get_reward_depts(user);

            for (index, reward) in rewads.iter().enumerate() {
                pending[index] = reward - user.reward_debts.get_unchecked(index as u32);
                if pending[index] > 0 {
                    user.reward_debts.set(index as u32, *reward);
                }
            }

            return Ok(pending);
        }

        Ok(pending)
    }

    // TODO
    pub(crate) fn add_rewards(&mut self, mut reward_amount: u128, token: Token) {
        if self.total_lp_amount > 0 {
            let admin_fee_rewards = reward_amount * self.admin_fee_share_bp / Pool::BP;
            reward_amount -= admin_fee_rewards;

            let acc_reward_per_share_p = self.acc_rewards_per_share_p.get_unchecked(token as u32)
                + (reward_amount << Pool::P) / self.total_lp_amount;

            self.acc_rewards_per_share_p
                .set(token as u32, acc_reward_per_share_p);

            self.admin_fee_amount += admin_fee_rewards;
        }
    }

    pub fn get_pending(&self, user: &UserDeposit) -> [u128; 2] {
        let mut pendings = self.acc_rewards_per_share_p.iter().enumerate().map(
            |(index, acc_reward_per_share_p)| {
                ((user.lp_amount * acc_reward_per_share_p) >> Pool::P)
                    - user.reward_debts.get_unchecked(index as u32)
            },
        );

        [pendings.next().unwrap(), pendings.next().unwrap()]
    }

    pub fn get_reward_depts(&self, user: &UserDeposit) -> [u128; 2] {
        let mut reward_debts = self
            .acc_rewards_per_share_p
            .iter()
            .map(|acc_reward_per_share_p| (user.lp_amount * acc_reward_per_share_p) >> Pool::P);

        [reward_debts.next().unwrap(), reward_debts.next().unwrap()]
    }

    // y = (sqrt(x(4AD³ + x (4A(D - x) - D )²)) + x (4A(D - x) - D ))/8Ax
    pub fn get_y(&self, native_x: u128) -> u128 {
        let d = self.get_current_d();

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
        self.get_d(
            self.token_balances.get_unchecked(0),
            self.token_balances.get_unchecked(1),
        )
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
