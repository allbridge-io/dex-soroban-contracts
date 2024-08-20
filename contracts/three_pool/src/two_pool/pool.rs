use ethnum::U256;
use shared::{
    require,
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{Address, Env};

use crate::{
    common::{Pool, PoolStorage, PoolView, WithdrawAmount},
    storage::{
        sized_array::{SizedAddressArray, SizedDecimalsArray, SizedU128Array},
        user_deposit::UserDeposit,
    },
};

use super::{
    events::{Deposit, RewardsClaimed, Withdraw},
    token::Token,
    two_pool::TwoPool,
};

impl Pool<2> for TwoPool {
    type Deposit = Deposit;
    type RewardsClaimed = RewardsClaimed;
    type Withdraw = Withdraw;
    type Token = Token;

    const BP: u128 = 10000;

    const MAX_A: u128 = 60;
    const MAX_TOKEN_BALANCE: u128 = 2u128.pow(40);
    const SYSTEM_PRECISION: u32 = 3;

    const P: u128 = 48;

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
    ) -> Result<(WithdrawAmount<2>, SizedU128Array), Error> {
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
