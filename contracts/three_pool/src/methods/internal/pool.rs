use ethnum::I256;
use shared::{
    require,
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{Address, Env};

use crate::{
    common::PoolView,
    storage::{
        common::Token,
        pool::ThreePool,
        sized_array::{SizedAddressArray, SizedDecimalsArray},
        user_deposit::UserDeposit,
    },
};
use crate::{
    common::{Pool, PoolStorage},
    storage::sized_array::SizedU128Array,
};

use crate::common::WithdrawAmount;

impl Pool<3> for ThreePool {
    type Deposit = crate::events::three_pool_events::Deposit;
    type RewardsClaimed = crate::events::three_pool_events::RewardsClaimed;
    type Withdraw = crate::events::three_pool_events::Withdraw;

    fn from_init_params(
        env: &Env,
        a: u128,
        tokens: [Address; 3],
        decimals: [u32; 3],
        fee_share_bp: u128,
        admin_fee_share_bp: u128,
    ) -> Self {
        ThreePool {
            a,

            fee_share_bp,
            admin_fee_share_bp,
            total_lp_amount: 0,

            tokens: SizedAddressArray::from_array(env, tokens),
            tokens_decimals: SizedDecimalsArray::from_array(env, decimals),
            token_balances: SizedU128Array::default_val::<3>(env),
            acc_rewards_per_share_p: SizedU128Array::default_val::<3>(env),
            admin_fee_amount: SizedU128Array::default_val::<3>(env),
        }
    }

    fn get_current_d(&self) -> Result<u128, Error> {
        self.get_d([
            self.token_balances.get(0usize),
            self.token_balances.get(1usize),
            self.token_balances.get(2usize),
        ])
    }

    fn get_d(&self, values: [u128; 3]) -> Result<u128, Error> {
        let x = I256::from(values[0]);
        let y = I256::from(values[1]);
        let z = I256::from(values[2]);
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

    fn get_y(&self, values: [u128; 3]) -> Result<u128, Error> {
        let x = I256::from(values[0]);
        let z = I256::from(values[1]);
        let d = I256::from(values[2]);
        let a = I256::from(self.a);
        let a27 = a * 27;

        let b = x + z - d + d / a27;
        let c = d.pow(4) / (-27 * a27 * x * z);
        Ok(((-b + sqrt(&(b.pow(2) - 4 * c).unsigned_abs()).as_i256()) / 2).as_u128())
    }

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
}

#[allow(clippy::inconsistent_digit_grouping)]
#[cfg(test)]
mod tests {
    extern crate std;

    use shared::{soroban_data::SimpleSorobanData, Error};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    use crate::storage::pool::ThreePool;
    use crate::storage::sized_array::SizedU128Array;

    use super::Pool;

    #[contract]
    pub struct TestPool;

    #[contractimpl]
    impl TestPool {
        pub fn init(env: Env) {
            let token_a = Address::generate(&env);
            let token_b = Address::generate(&env);
            let token_c = Address::generate(&env);
            ThreePool::from_init_params(&env, 20, [token_a, token_b, token_c], [7, 7, 7], 100, 1)
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
            ThreePool::get(&env)?.get_y([x, z, d])
        }
        pub fn get_d(env: Env, x: u128, y: u128, z: u128) -> Result<u128, Error> {
            ThreePool::get(&env)?.get_d([x, y, z])
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
