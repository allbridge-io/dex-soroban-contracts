use core::cmp::Ordering;

use ethnum::U256;
use shared::{require, utils::num::*, Error};
use soroban_sdk::{contracttype, Address, Env};

use crate::storage::{
    double_values::DoubleU128,
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

pub struct ReceiveAmount {
    pub token_from_new_balance: u128,
    pub token_to_new_balance: u128,
    pub output: u128,
    pub fee: u128,
}

impl Pool {
    const MAX_TOKEN_BALANCE: u128 = 2u128.pow(40);
    const SYSTEM_PRECISION: u32 = 3;
    const BP: u128 = 10000;

    pub const P: u128 = 48;

    pub fn get_receive_amount(&self, input: u128, token_from: Token) -> ReceiveAmount {
        let token_to = token_from.opposite();
        let d0 = self.total_lp_amount;
        let input_sp = self.amount_to_system_precision(input, self.tokens_decimals[token_from]);
        let mut output = 0;

        let token_from_new_balance = self.token_balances[token_from] + input_sp;

        let token_to_new_amount = self.get_y(token_from_new_balance, d0);
        if self.token_balances[token_to] > token_to_new_amount {
            output = self.amount_from_system_precision(
                self.token_balances[token_to] - token_to_new_amount,
                self.tokens_decimals[token_to],
            );
        }
        let fee = output * self.fee_share_bp / Self::BP;

        output -= fee;

        ReceiveAmount {
            token_from_new_balance,
            token_to_new_balance: token_to_new_amount,
            output,
            fee,
        }
    }

    pub fn get_send_amount(&self, output: u128, token_to: Token) -> (u128, u128) {
        let token_from = token_to.opposite();
        let d0 = self.total_lp_amount;
        let fee = output * self.fee_share_bp / (Self::BP - self.fee_share_bp);
        let output_with_fee = output + fee;
        let output_sp =
            self.amount_to_system_precision(output_with_fee, self.tokens_decimals[token_to]);
        let mut input = 0;

        let token_to_new_balance = self.token_balances[token_to] - output_sp;

        let token_from_new_amount = self.get_y(token_to_new_balance, d0);
        if self.token_balances[token_from] < token_from_new_amount {
            input = self.amount_from_system_precision(
                token_from_new_amount - self.token_balances[token_from],
                self.tokens_decimals[token_from],
            );
        }

        (input, fee)
    }

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
        let receive_amount = self.get_receive_amount(amount, token_from);

        self.get_token(env, token_from)
            .transfer(&sender, &current_contract, &(amount as i128));

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
            &(receive_amount.output as i128),
        );

        Ok((receive_amount.output, receive_amount.fee))
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
        let d0 = self.total_lp_amount;

        let amounts_sp = DoubleU128::from((
            self.amount_to_system_precision(amounts[0], self.tokens_decimals[0]),
            self.amount_to_system_precision(amounts[1], self.tokens_decimals[1]),
        ));
        let total_amount: u128 = amounts_sp.to_array().iter().sum();
        require!(total_amount > 0, Error::ZeroAmount);

        for (index, amount) in amounts.to_array().into_iter().enumerate() {
            if amount == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &sender,
                &current_contract,
                &(amount as i128),
            );

            self.token_balances[index] += amounts_sp[index];
        }

        let d1 = self.get_current_d();

        require!(d1 > d0, Error::Forbidden);

        let lp_amount = d1 - d0;

        require!(lp_amount >= min_lp_amount, Error::Slippage);
        require!(
            self.token_balances.to_array().iter().sum::<u128>() < Self::MAX_TOKEN_BALANCE,
            Error::PoolOverflow
        );

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
        let d0 = self.total_lp_amount;
        let old_balances = self.token_balances.clone();
        let rewards_amounts = self.withdraw_lp(user, lp_amount)?;
        let mut amounts = DoubleU128::default();

        let d1 = d0 - lp_amount;
        let (more, less) = if self.token_balances[0] > self.token_balances[1] {
            (0, 1)
        } else {
            (1, 0)
        };
        let more_token_amount = self.token_balances[more] * lp_amount / d0;
        let less_token_amount = self.token_balances[less]
            - self.get_y(self.token_balances[more] - more_token_amount, d1);

        for (index, token_amount) in [(more, more_token_amount), (less, less_token_amount)] {
            amounts[index] = token_amount;
            self.token_balances[index] -= token_amount;
            let token_amount =
                self.amount_from_system_precision(token_amount, self.tokens_decimals[index]);
            let withdraw_amount = token_amount + rewards_amounts[index];

            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &(withdraw_amount as i128),
            );
        }

        let new_balances = self.token_balances.clone();
        let d1 = self.total_lp_amount;

        require!(
            new_balances[0] < old_balances[0] && new_balances[1] < old_balances[1] && d1 < d0,
            Error::ZeroChanges
        );

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

        let rewards = self.get_reward_depts(user);

        for (index, reward) in rewards.to_array().into_iter().enumerate() {
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

#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    use shared::{soroban_data::SimpleSorobanData, Error};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    use crate::storage::{
        double_values::DoubleU128,
        pool::{Pool, Token},
    };

    #[contract]
    pub struct TestPool;

    #[contractimpl]
    impl TestPool {
        pub fn init(env: Env) {
            let token_a = Address::generate(&env);
            let token_b = Address::generate(&env);
            Pool::from_init_params(20, token_a, token_b, (7, 7), 100, 1).save(&env);
        }

        pub fn set_balances(env: Env, new_balances: (u128, u128)) -> Result<(), Error> {
            Pool::update(&env, |pool| {
                pool.token_balances = DoubleU128::from(new_balances);
                pool.total_lp_amount = pool.get_current_d();
                Ok(())
            })
        }

        pub fn get_receive_amount(
            env: Env,
            amount: u128,
            token_from: Token,
        ) -> Result<(u128, u128), Error> {
            let receive_amount = Pool::get(&env)?.get_receive_amount(amount, token_from);
            Ok((receive_amount.output, receive_amount.fee))
        }

        pub fn get_send_amount(
            env: Env,
            amount: u128,
            token_to: Token,
        ) -> Result<(u128, u128), Error> {
            Ok(Pool::get(&env)?.get_send_amount(amount, token_to))
        }
    }

    #[test]
    fn test() {
        let env = Env::default();

        let test_pool_id = env.register_contract(None, TestPool);
        let pool = TestPoolClient::new(&env, &test_pool_id);
        pool.init();
        pool.set_balances(&(200_000_000, 200_000_000));

        let input = 10_000_0000000u128;
        let (output, fee) = pool.get_receive_amount(&input, &Token::A);
        let (calc_input, calc_fee) = pool.get_send_amount(&output, &Token::B);

        println!("input: {}", input);
        println!("output: {}, fee: {}", output, fee);
        println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

        assert_eq!(input, calc_input);
        assert_eq!(fee, calc_fee);
    }

    #[test]
    fn test_disbalance() {
        let env = Env::default();

        let test_pool_id = env.register_contract(None, TestPool);
        let pool = TestPoolClient::new(&env, &test_pool_id);
        pool.init();
        pool.set_balances(&(200_000_000, 500_000_000));

        let input = 10_000_0000000u128;
        let (output, fee) = pool.get_receive_amount(&input, &Token::A);
        let (calc_input, calc_fee) = pool.get_send_amount(&output, &Token::B);

        println!("input: {}", input);
        println!("output: {}, fee: {}", output, fee);
        println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

        assert_eq!(input, calc_input);
        assert_eq!(fee, calc_fee);
    }
}
