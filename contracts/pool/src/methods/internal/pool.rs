use core::cmp::Ordering;

use ethnum::I256;
use shared::{
    require,
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{Address, Env};

use crate::storage::{
    common::{Token},
    pool::Pool,
    user_deposit::UserDeposit,
};
use crate::storage::triple_values::TripleU128;

use super::pool_view::WithdrawAmount;

impl Pool {
    pub const BP: u128 = 10000;

    pub(crate) const MAX_A: u128 = 60;
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
        token_from: Token,
        token_to: Token,
    ) -> Result<(u128, u128), Error> {
        if amount == 0 {
            return Ok((0, 0));
        }

        let current_contract = env.current_contract_address();
        let receive_amount = self.get_receive_amount(amount, token_from, token_to)?;

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
        amounts: TripleU128,
        sender: Address,
        user_deposit: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<(TripleU128, u128), Error> {
        let current_contract = env.current_contract_address();

        if self.total_lp_amount == 0 {
            require!(amounts.data.0 == amounts.data.1 && amounts.data.0 == amounts.data.2 && amounts.data.1 == amounts.data.2, Error::InvalidFirstDeposit);
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
    ) -> Result<(WithdrawAmount, TripleU128), Error> {
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
    ) -> Result<TripleU128, Error> {
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
    ) -> Result<TripleU128, Error> {
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
    ) -> Result<TripleU128, Error> {
        let mut pending = TripleU128::default();

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

    pub fn get_pending(&self, user_deposit: &UserDeposit) -> TripleU128 {
        if user_deposit.lp_amount == 0 {
            return TripleU128::default();
        }

        let rewards = self.get_reward_debts(user_deposit);

        TripleU128::from((
            rewards[0] - user_deposit.reward_debts[0],
            rewards[1] - user_deposit.reward_debts[1],
            rewards[2] - user_deposit.reward_debts[2],
        ))
    }

    pub fn get_reward_debts(&self, user_deposit: &UserDeposit) -> TripleU128 {
        TripleU128::from((
            (user_deposit.lp_amount * self.acc_rewards_per_share_p[0]) >> Pool::P,
            (user_deposit.lp_amount * self.acc_rewards_per_share_p[1]) >> Pool::P,
            (user_deposit.lp_amount * self.acc_rewards_per_share_p[2]) >> Pool::P,
        ))
    }


    pub fn get_y(&self, native_x: u128, native_z: u128, d: u128) -> Result<u128, Error> {
        let a27 = self.a * 27;
        let int_a27: i128 = safe_cast(a27)?;

        let int_d: i128 = safe_cast(d)?;
        let int_native_x: i128 = safe_cast(native_x)?;
        let int_native_z: i128 = safe_cast(native_z)?;


        let b: I256 = I256::new(int_native_x + int_native_z - int_d) + I256::new(int_d / int_a27);
        let c: I256 = I256::new(int_d).pow(4) / (27 * int_a27 * int_native_x * int_native_z) * -1;
        Ok(((-1 * b + sqrt(&(b.pow(2) - 4 * c).unsigned_abs()).as_i256()) / 2).as_u128())
    }

    pub fn get_current_d(&self) -> Result<u128, Error> {
        self.get_d(self.token_balances[0], self.token_balances[1], self.token_balances[2])
    }

    fn f(&self, x128: u128, y128: u128, z128: u128, d128: u128) -> I256 {
        let x = I256::from(x128);
        let y = I256::from(y128);
        let z = I256::from(z128);
        let d = I256::from(d128);
        let a = I256::from(self.a);
        27 * a * (x + y + z) - (27 * a * d - d) - d.pow(4) / (27 * x * y * z)
    }

    fn df(&self, x128: u128, y128: u128, z128: u128, d128: u128) -> I256 {
        let x = I256::from(x128);
        let y = I256::from(y128);
        let z = I256::from(z128);
        let d = I256::from(d128);
        let a = I256::from(self.a);
        -4 * d.pow(3) / (27 * x * y * z) - 27 * a + 1
    }

    pub fn get_d(&self, x: u128, y: u128, z: u128) -> Result<u128, Error> {
        let mut d = x + y + z;
        loop {
            let f = self.f(x, y, z, d);
            let df = self.df(x, y, z, d);
            if f.abs() < df.abs() {
                break;
            }
            d = ((d as i128) - (f / df).as_i128()) as u128;
        }

        Ok(d)
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


#[allow(clippy::inconsistent_digit_grouping)]
#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    use shared::{soroban_data::SimpleSorobanData, Error};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    use crate::storage::{pool::Pool};
    use crate::storage::triple_values::TripleU128;

    #[contract]
    pub struct TestPool;

    #[contractimpl]
    impl TestPool {
        pub fn init(env: Env) {
            let token_a = Address::generate(&env);
            let token_b = Address::generate(&env);
            let token_c = Address::generate(&env);
            Pool::from_init_params(20, token_a, token_b, token_c, (7, 7, 7), 100, 1).save(&env);
        }

        pub fn set_balances(env: Env, new_balances: (u128, u128, u128)) -> Result<(), Error> {
            Pool::update(&env, |pool| {
                pool.token_balances = TripleU128::from(new_balances);
                pool.total_lp_amount = pool.get_current_d()?;
                Ok(())
            })
        }

        pub fn get_y(
            env: Env,
            x: u128,
            z: u128,
            d: u128,
        ) -> Result<u128, Error> {
            Pool::get(&env)?.get_y(x, z, d)
        }
        pub fn get_d(
            env: Env,
            x: u128,
            y: u128,
            z: u128,
        ) -> Result<u128, Error> {
            Pool::get(&env)?.get_d(x, y, z)
        }
    }

    #[test]
    fn test_get_y() {
        let env = Env::default();

        let test_pool_id = env.register_contract(None, TestPool);
        let pool = TestPoolClient::new(&env, &test_pool_id);
        pool.init();
        let result = pool.get_y(&1_000_000, &1_000_000, &3_000_000);

        println!("result: {}", result);

        assert_eq!(result, 1_000_000);
    }
    #[test]
    fn test_get_d() {
        let env = Env::default();

        let test_pool_id = env.register_contract(None, TestPool);
        let pool = TestPoolClient::new(&env, &test_pool_id);
        pool.init();
        let result = pool.get_d(&2_000_000, &256_364, &5_000_000);

        println!("result: {}", result);

        assert_eq!(result, 7_197_881);
    }
}
