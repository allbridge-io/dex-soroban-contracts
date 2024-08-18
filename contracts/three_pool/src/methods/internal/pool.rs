use core::cmp::Ordering;

use ethnum::I256;
use shared::{
    require,
    soroban_data::SimpleSorobanData,
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{
    token::{self, TokenClient},
    Address, Env,
};

use crate::storage::sized_array::{SizedAddressArray, SizedDecimalsArray, SizedU128Array};
use crate::storage::{common::Token, pool::ThreePool, user_deposit::UserDeposit};

use super::pool_view::WithdrawAmount;

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
    fn get_token_by_index(&self, env: &Env, index: usize) -> TokenClient<'_> {
        token::Client::new(env, &self.tokens().get(index))
    }

    #[inline]
    fn get_token(&self, env: &Env, token: Token) -> TokenClient<'_> {
        self.get_token_by_index(env, token as usize)
    }
}

pub trait Pool: PoolStorage + SimpleSorobanData {
    const BP: u128 = 10000;

    const MAX_A: u128 = 60;
    const MAX_TOKEN_BALANCE: u128 = 2u128.pow(40);
    const SYSTEM_PRECISION: u32 = 3;

    const P: u128 = 48;

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
        let mut pending = SizedU128Array::default_val(env);

        if user_deposit.lp_amount == 0 {
            return Ok(pending);
        }

        let rewards = self.get_reward_debts(env, user_deposit);

        for (index, reward) in rewards.iter().enumerate() {
            pending.set(index, reward - user_deposit.reward_debts.get(index));

            if pending.get(index) > 0 {
                user_deposit.reward_debts.set(index, reward);

                self.get_token_by_index(env, index).transfer(
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
            let admin_fee_rewards = reward_amount * self.admin_fee_share_bp() / ThreePool::BP;
            reward_amount -= admin_fee_rewards;

            let total_lp_amount = self.total_lp_amount();
            self.acc_rewards_per_share_p_mut()
                .add_by_token(token, (reward_amount << ThreePool::P) / total_lp_amount);
            self.admin_fee_amount_mut()
                .add_by_token(token, admin_fee_rewards);
        }
    }

    fn get_pending(&self, env: &Env, user_deposit: &UserDeposit) -> SizedU128Array {
        if user_deposit.lp_amount == 0 {
            return SizedU128Array::default_val(env);
        }

        let rewards = self.get_reward_debts(env, user_deposit);

        rewards - user_deposit.reward_debts.clone()
    }

    fn get_reward_debts(&self, env: &Env, user_deposit: &UserDeposit) -> SizedU128Array {
        let mut v = SizedU128Array::default_val(env);

        for (index, acc_rewards_per_share_p) in self.acc_rewards_per_share_p().iter().enumerate() {
            let new_acc_rewards =
                (user_deposit.lp_amount * acc_rewards_per_share_p) >> ThreePool::P;
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

impl PoolStorage for ThreePool {
    fn a(&self) -> u128 {
        self.a
    }
    fn fee_share_bp(&self) -> u128 {
        self.fee_share_bp
    }
    fn admin_fee_share_bp(&self) -> u128 {
        self.admin_fee_share_bp
    }
    fn total_lp_amount(&self) -> u128 {
        self.total_lp_amount
    }
    fn tokens(&self) -> &SizedAddressArray {
        &self.tokens
    }
    fn tokens_decimals(&self) -> &SizedDecimalsArray {
        &self.tokens_decimals
    }
    fn token_balances(&self) -> &SizedU128Array {
        &self.token_balances
    }
    fn acc_rewards_per_share_p(&self) -> &SizedU128Array {
        &self.acc_rewards_per_share_p
    }
    fn admin_fee_amount(&self) -> &SizedU128Array {
        &self.admin_fee_amount
    }

    fn a_mut(&mut self) -> &mut u128 {
        &mut self.a
    }
    fn fee_share_bp_mut(&mut self) -> &mut u128 {
        &mut self.fee_share_bp
    }
    fn admin_fee_share_bp_mut(&mut self) -> &mut u128 {
        &mut self.admin_fee_share_bp
    }
    fn total_lp_amount_mut(&mut self) -> &mut u128 {
        &mut self.total_lp_amount
    }
    fn tokens_mut(&mut self) -> &mut SizedAddressArray {
        &mut self.tokens
    }
    fn tokens_decimals_mut(&mut self) -> &mut SizedDecimalsArray {
        &mut self.tokens_decimals
    }
    fn token_balances_mut(&mut self) -> &mut SizedU128Array {
        &mut self.token_balances
    }
    fn acc_rewards_per_share_p_mut(&mut self) -> &mut SizedU128Array {
        &mut self.acc_rewards_per_share_p
    }
    fn admin_fee_amount_mut(&mut self) -> &mut SizedU128Array {
        &mut self.admin_fee_amount
    }
}

impl Pool for ThreePool {
    #[allow(clippy::too_many_arguments)]
    fn swap(
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

        self.token_balances
            .set_by_token(token_from, receive_amount.token_from_new_balance);
        self.token_balances
            .set_by_token(token_to, receive_amount.token_to_new_balance);

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
            require!(
                amounts.get(0) == amounts.get(1)
                    && amounts.get(0) == amounts.get(2)
                    && amounts.get(1) == amounts.get(2),
                Error::InvalidFirstDeposit
            );
        }

        let deposit_amount = self.get_deposit_amount(env, amounts.clone())?;
        self.token_balances = SizedU128Array::from_vec(deposit_amount.new_token_balances);

        require!(deposit_amount.lp_amount >= min_lp_amount, Error::Slippage);

        for (index, amount) in amounts.iter().enumerate() {
            if amount == 0 {
                continue;
            }

            self.get_token_by_index(env, index).transfer(
                &sender,
                &current_contract,
                &safe_cast(amount)?,
            );
        }

        let rewards = self.deposit_lp(env, user_deposit, deposit_amount.lp_amount)?;

        for (index, reward) in rewards.iter().enumerate() {
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

    fn withdraw(
        &mut self,
        env: &Env,
        sender: Address,
        user_deposit: &mut UserDeposit,
        lp_amount: u128,
    ) -> Result<(WithdrawAmount, SizedU128Array), Error> {
        let current_contract = env.current_contract_address();
        let d0 = self.total_lp_amount;
        let old_balances = self.token_balances.clone();
        let withdraw_amount = self.get_withdraw_amount(env, lp_amount)?;
        let rewards_amounts = self.withdraw_lp(env, user_deposit, lp_amount)?;

        for index in withdraw_amount.indexes {
            let token_amount = self.amount_from_system_precision(
                withdraw_amount.amounts.get(index),
                self.tokens_decimals.get(index),
            );
            let token_amount = token_amount + rewards_amounts.get(index);

            self.add_rewards(withdraw_amount.fees.get(index), index.into());
            self.get_token_by_index(env, index).transfer(
                &current_contract,
                &sender,
                &safe_cast(token_amount)?,
            );
        }

        self.token_balances = withdraw_amount.new_token_balances.clone();
        let d1 = self.total_lp_amount;

        require!(
            self.token_balances.get(0) < old_balances.get(0)
                && self.token_balances.get(1) < old_balances.get(1)
                && d1 < d0,
            Error::ZeroChanges
        );

        Ok((withdraw_amount, rewards_amounts))
    }
}

impl ThreePool {
    pub fn get_y(&self, x128: u128, z128: u128, d128: u128) -> Result<u128, Error> {
        let x = I256::from(x128);
        let z = I256::from(z128);
        let d = I256::from(d128);
        let a = I256::from(self.a);
        let a27 = a * 27;

        let b = x + z - d + d / a27;
        let c = d.pow(4) / (-27 * a27 * x * z);
        Ok(((-b + sqrt(&(b.pow(2) - 4 * c).unsigned_abs()).as_i256()) / 2).as_u128())
    }

    pub fn get_current_d(&self) -> Result<u128, Error> {
        self.get_d(
            self.token_balances.get(0),
            self.token_balances.get(1),
            self.token_balances.get(2),
        )
    }

    pub fn get_d(&self, x128: u128, y128: u128, z128: u128) -> Result<u128, Error> {
        let x = I256::from(x128);
        let y = I256::from(y128);
        let z = I256::from(z128);
        let a = I256::from(self.a);

        let mut d = x + y + z;
        loop {
            let f = 27 * a * (x + y + z) - (27 * a * d - d) - d.pow(4) / (27 * x * y * z);
            let df = -4 * d.pow(3) / (27 * x * y * z) - 27 * a + 1;
            if f.abs() < df.abs() {
                break;
            }
            d -= f / df;
        }

        Ok(d.as_u128())
    }
}

#[allow(clippy::inconsistent_digit_grouping)]
#[cfg(test)]
mod tests {
    extern crate std;

    use shared::{soroban_data::SimpleSorobanData, Error};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    use crate::storage::pool::ThreePool;
    use crate::storage::sized_array::SizedU128Array;

    #[contract]
    pub struct TestPool;

    #[contractimpl]
    impl TestPool {
        pub fn init(env: Env) {
            let token_a = Address::generate(&env);
            let token_b = Address::generate(&env);
            let token_c = Address::generate(&env);
            ThreePool::from_init_params(&env, 20, token_a, token_b, token_c, [7, 7, 7], 100, 1)
                .save(&env);
        }

        pub fn set_balances(env: Env, new_balances: (u128, u128, u128)) -> Result<(), Error> {
            ThreePool::update(&env, |pool| {
                let (a, b, c) = new_balances;
                pool.token_balances = SizedU128Array::from_array(&env, [a, b, c]);
                pool.total_lp_amount = pool.get_current_d()?;
                Ok(())
            })
        }

        pub fn get_y(env: Env, x: u128, z: u128, d: u128) -> Result<u128, Error> {
            ThreePool::get(&env)?.get_y(x, z, d)
        }
        pub fn get_d(env: Env, x: u128, y: u128, z: u128) -> Result<u128, Error> {
            ThreePool::get(&env)?.get_d(x, y, z)
        }
    }

    #[test]
    fn test_get_y() {
        let env = Env::default();

        let test_pool_id = env.register_contract(None, TestPool);
        let pool = TestPoolClient::new(&env, &test_pool_id);
        pool.init();

        assert_eq!(pool.get_y(&1_000_000, &1_000_000, &3_000_000), 1_000_000);

        let n = 100_000_000_000_000_000;
        let big_d = 157_831_140_060_220_325;
        let mid_d = 6_084_878_857_843_302;
        assert_eq!(pool.get_y(&n, &n, &(n * 3)), n);
        assert_eq!(pool.get_y(&n, &(n / 1_000), &big_d), n - 1);
        assert_eq!(pool.get_y(&n, &n, &big_d), n / 1_000 - 1);
        assert_eq!(pool.get_y(&n, &(n / 1_000), &mid_d), n / 1_000_000 - 1);
        assert_eq!(pool.get_y(&n, &(n / 1_000_000), &mid_d), n / 1_000 - 1);
        assert_eq!(pool.get_y(&(n / 1_000), &(n / 1_000_000), &mid_d), n - 14);
    }

    #[test]
    fn test_get_d() {
        let env = Env::default();

        let test_pool_id = env.register_contract(None, TestPool);
        let pool = TestPoolClient::new(&env, &test_pool_id);
        pool.init();

        assert_eq!(pool.get_d(&2_000_000, &256_364, &5_000_000), 7_197_881);

        let n = 100_000_000_000_000_000;
        let big_d = 157_831_140_060_220_325;
        assert_eq!(pool.get_d(&n, &n, &n), n * 3);
        assert_eq!(pool.get_d(&n, &n, &(n / 1_000)), big_d);
        assert_eq!(
            pool.get_d(&n, &(n / 1_000), &(n / 1_000_000)),
            6_084_878_857_843_302
        );
    }
}
