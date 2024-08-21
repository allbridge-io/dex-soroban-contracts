use ethnum::U256;
use shared::{
    require,
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{Address, Env};

use crate::{
    pool::{Pool, PoolMath, PoolStorage, ReceiveAmount, WithdrawAmount},
    storage::{
        sized_array::{SizedAddressArray, SizedDecimalsArray, SizedU128Array},
        user_deposit::UserDeposit,
    },
};

use super::{pool::TwoPool, token::TwoToken};

impl PoolMath<2> for TwoPool {
    // y = (sqrt(x(4AD³ + x (4A(D - x) - D )²)) + x (4A(D - x) - D ))/8Ax
    fn get_y(&self, values: [u128; 2]) -> Result<u128, Error> {
        let native_x = values[0];
        let d = values[1];

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
        let sqrt_sum = safe_cast::<_, i128>(sqrt(&part2).as_u128())? + (int_native_x * part1);
        // (sqrt(part2) + x(part1)) / 8Ax)
        Ok(safe_cast::<_, u128>(sqrt_sum)? / ((self.a << 3) * native_x))
    }

    fn get_current_d(&self) -> Result<u128, Error> {
        self.get_d([
            self.token_balances.get(0usize),
            self.token_balances.get(1usize),
        ])
    }

    fn get_d(&self, values: [u128; 2]) -> Result<u128, Error> {
        let x = values[0];
        let y = values[1];
        let xy: u128 = x * y;
        // Axy(x+y)
        let p1 = U256::new(self.a * (x + y) * xy);

        // xy(4A - 1) / 3
        let p2 = U256::new(xy * ((self.a << 2) - 1) / 3);

        // sqrt(p1² + p2³)
        let p3 = sqrt(&(square(p1)? + cube(p2)?));

        // cbrt(p1 + p3) + cbrt(p1 - p3)
        let mut d = cbrt(&(p1.checked_add(p3).ok_or(Error::U256Overflow)?))?;
        if p3.gt(&p1) {
            d -= cbrt(&(p3 - p1))?;
        } else {
            d += cbrt(&(p1 - p3))?;
        }

        Ok(d << 1)
    }
}

impl Pool<2> for TwoPool {
    type Token = TwoToken;

    fn from_init_params(
        env: &Env,
        a: u128,
        tokens: [Address; 2],
        decimals: [u32; 2],
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self {
        TwoPool {
            a,

            fee_share_bp,
            admin_fee_share_bp,
            total_lp_amount: 0,

            tokens: SizedAddressArray::from_array(env, tokens),
            tokens_decimals: SizedDecimalsArray::from_array(env, decimals),
            token_balances: SizedU128Array::default_val::<2>(env),
            acc_rewards_per_share_p: SizedU128Array::default_val::<2>(env),
            admin_fee_amount: SizedU128Array::default_val::<2>(env),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn swap(
        &mut self,
        env: &Env,
        sender: Address,
        recipient: Address,
        amount: u128,
        receive_amount_min: u128,
        token_from: TwoToken,
        token_to: TwoToken,
    ) -> Result<(u128, u128), Error> {
        if amount == 0 {
            return Ok((0, 0));
        }

        let current_contract = env.current_contract_address();
        let receive_amount = self.get_receive_amount(amount, token_from, token_to)?;

        self.get_token(env, token_from)
            .transfer(&sender, &current_contract, &safe_cast(amount)?);

        self.token_balances
            .set(token_from, receive_amount.token_from_new_balance);
        self.token_balances
            .set(token_to, receive_amount.token_to_new_balance);

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

    fn deposit(
        &mut self,
        env: &Env,
        amounts: SizedU128Array,
        sender: Address,
        user_deposit: &mut UserDeposit,
        min_lp_amount: u128,
    ) -> Result<(SizedU128Array, u128), Error> {
        let current_contract = env.current_contract_address();

        if self.total_lp_amount == 0 {
            let first = amounts.get(0usize);
            let is_deposit_valid = amounts.iter().all(|v| v == first);
            require!(is_deposit_valid, Error::InvalidFirstDeposit);
        }

        let deposit_amount = self.get_deposit_amount(env, amounts.clone())?;
        self.token_balances = SizedU128Array::from_vec(deposit_amount.new_token_balances);

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
    ) -> Result<(WithdrawAmount<2>, SizedU128Array), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.total_lp_amount;
        let old_balances = self.token_balances.clone();
        let withdraw_amount = self.get_withdraw_amount(env, lp_amount)?;
        let rewards_amounts = self.withdraw_lp(env, user_deposit, lp_amount)?;

        for index in withdraw_amount.indexes {
            let token_amount =
                self.amount_from_system_precision(withdraw_amount.amounts.get(index), index);
            let token_amount = token_amount + rewards_amounts.get(index);

            self.add_rewards(withdraw_amount.fees.get(index), index.into());
            self.get_token(env, index).transfer(
                &current_contract,
                &sender,
                &safe_cast(token_amount)?,
            );
        }

        self.token_balances = withdraw_amount.new_token_balances.clone();
        let d1 = self.total_lp_amount;

        require!(
            self.token_balances.get(0usize) < old_balances.get(0usize)
                && self.token_balances.get(1usize) < old_balances.get(1usize)
                && d1 < d0,
            Error::ZeroChanges
        );

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
        let mut pending = SizedU128Array::default_val::<2>(env);

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

    fn add_rewards(&mut self, mut reward_amount: u128, token: Self::Token) {
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
            return SizedU128Array::default_val::<2>(env);
        }

        let rewards = self.get_reward_debts(env, user_deposit);

        rewards - user_deposit.reward_debts.clone()
    }

    fn get_reward_debts(&self, env: &Env, user_deposit: &UserDeposit) -> SizedU128Array {
        let mut v = SizedU128Array::default_val::<2>(env);

        for (index, acc_rewards_per_share_p) in self.acc_rewards_per_share_p().iter().enumerate() {
            let new_acc_rewards = (user_deposit.lp_amount * acc_rewards_per_share_p) >> Self::P;
            v.set(index, new_acc_rewards);
        }

        v
    }

    fn get_receive_amount(
        &self,
        input: u128,
        token_from: Self::Token,
        _token_to: Self::Token,
    ) -> Result<ReceiveAmount, Error> {
        let token_to = token_from.opposite();
        let d0 = self.total_lp_amount;
        let input_sp = self.amount_to_system_precision(input, token_from);
        let mut output = 0;

        let token_from_new_balance = self.token_balances.get(token_from) + input_sp;

        let token_to_new_balance = self.get_y([token_from_new_balance, d0])?;
        if self.token_balances.get(token_to) > token_to_new_balance {
            output = self.amount_from_system_precision(
                self.token_balances.get(token_to) - token_to_new_balance,
                token_to,
            );
        }
        let fee = output * self.fee_share_bp / Self::BP;

        output -= fee;

        Ok(ReceiveAmount {
            token_from_new_balance,
            token_to_new_balance,
            output,
            fee,
        })
    }

    fn get_send_amount(
        &self,
        output: u128,
        _token_from: Self::Token,
        token_to: Self::Token,
    ) -> Result<(u128, u128), Error> {
        let token_from = token_to.opposite();
        let d0 = self.total_lp_amount;
        let fee = output * self.fee_share_bp / (Self::BP - self.fee_share_bp);
        let output_with_fee = output + fee;
        let output_sp = self.amount_to_system_precision(output_with_fee, token_to);
        let mut input = 0;

        let token_to_new_balance = self.token_balances.get(token_to) - output_sp;

        let token_from_new_amount = self.get_y([token_to_new_balance, d0])?;
        if self.token_balances.get(token_from) < token_from_new_amount {
            input = self.amount_from_system_precision(
                token_from_new_amount - self.token_balances.get(token_from),
                token_from,
            );
        }

        Ok((input, fee))
    }

    fn get_withdraw_amount(&self, env: &Env, lp_amount: u128) -> Result<WithdrawAmount<2>, Error> {
        let d0 = self.total_lp_amount;
        let mut amounts = SizedU128Array::from_array(env, [0u128; 2]);

        let d1 = d0 - lp_amount;
        let (more, less) = if self.token_balances.get(0usize) > self.token_balances.get(1usize) {
            (0, 1)
        } else {
            (1, 0)
        };

        let more_token_amount_sp = self.token_balances.get(more) * lp_amount / d0;
        let y = self.get_y([self.token_balances.get(more) - more_token_amount_sp, d1])?;
        let less_token_amount_sp = self.token_balances.get(less) - y;

        let mut new_token_balances = self.token_balances.clone();
        let mut fees = SizedU128Array::from_array(env, [0u128; 2]);

        for (index, token_amount_sp) in [(more, more_token_amount_sp), (less, less_token_amount_sp)]
        {
            let token_amount = self.amount_from_system_precision(token_amount_sp, index);
            let fee = token_amount * self.fee_share_bp / Self::BP;

            let token_amount_sp = self.amount_to_system_precision(token_amount - fee, index);

            fees.set(index, fee);
            amounts.set(index, token_amount_sp);
            new_token_balances.sub(index, token_amount_sp);
        }

        Ok(WithdrawAmount {
            indexes: [more, less],
            fees,
            amounts,
            new_token_balances,
        })
    }

    fn amount_to_system_precision(&self, amount: u128, index: impl Into<usize>) -> u128 {
        let decimals = self.tokens_decimals().get(index);

        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            core::cmp::Ordering::Greater => {
                amount / (10u128.pow(decimals - Self::SYSTEM_PRECISION))
            }
            core::cmp::Ordering::Less => amount * (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            core::cmp::Ordering::Equal => amount,
        }
    }

    fn amount_from_system_precision(&self, amount: u128, index: impl Into<usize>) -> u128 {
        let decimals = self.tokens_decimals().get(index);

        match decimals.cmp(&Self::SYSTEM_PRECISION) {
            core::cmp::Ordering::Greater => {
                amount * (10u128.pow(decimals - Self::SYSTEM_PRECISION))
            }
            core::cmp::Ordering::Less => amount / (10u128.pow(Self::SYSTEM_PRECISION - decimals)),
            core::cmp::Ordering::Equal => amount,
        }
    }
}

#[allow(clippy::inconsistent_digit_grouping)]
#[cfg(test)]
mod view_tests {
    extern crate std;
    use std::println;

    use shared::{soroban_data::SimpleSorobanData, Error};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    use crate::{
        pool::{Pool, PoolMath},
        storage::sized_array::SizedU128Array,
        two_pool::{pool::TwoPool, token::TwoToken},
    };

    #[contract]
    pub struct TestPool;

    #[contractimpl]
    impl TestPool {
        pub fn init(env: Env) {
            let token_a = Address::generate(&env);
            let token_b = Address::generate(&env);
            TwoPool::from_init_params(&env, 20, [token_a, token_b], [7, 7], 100, 1).save(&env);
        }

        pub fn set_balances(env: Env, new_balances: (u128, u128)) -> Result<(), Error> {
            TwoPool::update(&env, |pool| {
                pool.token_balances =
                    SizedU128Array::from_array(&env, [new_balances.0, new_balances.1]);
                pool.total_lp_amount = pool.get_current_d()?;
                Ok(())
            })
        }

        pub fn get_receive_amount(
            env: Env,
            amount: u128,
            token_from: TwoToken,
            token_to: TwoToken,
        ) -> Result<(u128, u128), Error> {
            let receive_amount =
                TwoPool::get(&env)?.get_receive_amount(amount, token_from, token_to)?;
            Ok((receive_amount.output, receive_amount.fee))
        }

        pub fn get_send_amount(
            env: Env,
            amount: u128,
            token_from: TwoToken,
            token_to: TwoToken,
        ) -> Result<(u128, u128), Error> {
            TwoPool::get(&env)?.get_send_amount(amount, token_from, token_to)
        }
    }

    #[test]
    fn test() {
        let env = Env::default();

        let test_pool_id = env.register_contract(None, TestPool);
        let pool = TestPoolClient::new(&env, &test_pool_id);
        pool.init();
        pool.set_balances(&(200_000_000, 200_000_000));

        let input = 10_000_0000000_u128;
        let (output, fee) = pool.get_receive_amount(&input, &TwoToken::A, &TwoToken::B);
        let (calc_input, calc_fee) = pool.get_send_amount(&output, &TwoToken::A, &TwoToken::B);

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

        let input = 10_000_0000000_u128;
        let (output, fee) = pool.get_receive_amount(&input, &TwoToken::A, &TwoToken::B);
        let (calc_input, calc_fee) = pool.get_send_amount(&output, &TwoToken::A, &TwoToken::B);

        println!("input: {}", input);
        println!("output: {}, fee: {}", output, fee);
        println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

        assert_eq!(input, calc_input);
        assert_eq!(fee, calc_fee);
    }
}
