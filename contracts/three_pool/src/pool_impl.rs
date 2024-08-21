use ethnum::I256;
use shared::{
    require,
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{Address, Env};

use generic_pool::{
    pool::{Pool, PoolMath, PoolStorage, ReceiveAmount, WithdrawAmount},
    storage::{
        sized_array::{SizedAddressArray, SizedDecimalsArray, SizedU128Array},
        user_deposit::UserDeposit,
    },
};

use super::{pool::ThreePool, token::ThreeToken};

impl PoolMath<3> for ThreePool {
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
}

impl Pool<3> for ThreePool {
    type Token = ThreeToken;

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

    #[allow(clippy::too_many_arguments)]
    fn swap(
        &mut self,
        env: &Env,
        sender: Address,
        recipient: Address,
        amount: u128,
        receive_amount_min: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
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
    ) -> Result<(WithdrawAmount<3>, SizedU128Array), Error> {
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

    fn get_receive_amount(
        &self,
        input: u128,
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<ReceiveAmount, Error> {
        let d0 = self.total_lp_amount;
        let input_sp = self.amount_to_system_precision(input, token_from);
        let mut output = 0;

        let token_from_new_balance = self.token_balances.get(token_from) + input_sp;
        let token_third = token_from.third(token_to);

        let token_to_new_balance = self.get_y([
            token_from_new_balance,
            self.token_balances.get(token_third),
            d0,
        ])?;
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
        token_from: ThreeToken,
        token_to: ThreeToken,
    ) -> Result<(u128, u128), Error> {
        let d0 = self.total_lp_amount;
        let fee = output * self.fee_share_bp / (Self::BP - self.fee_share_bp);
        let output_with_fee = output + fee;
        let output_sp = self.amount_to_system_precision(output_with_fee, token_to);
        let mut input = 0;

        let token_to_new_balance = self.token_balances.get(token_to) - output_sp;
        let token_third = token_from.third(token_to);

        let token_from_new_amount = self.get_y([
            token_to_new_balance,
            self.token_balances.get(token_third),
            d0,
        ])?;
        if self.token_balances.get(token_from) < token_from_new_amount {
            input = self.amount_from_system_precision(
                token_from_new_amount - self.token_balances.get(token_from),
                token_from,
            );
        }

        Ok((input, fee))
    }

    fn get_withdraw_amount(&self, env: &Env, lp_amount: u128) -> Result<WithdrawAmount<3>, Error> {
        let d0 = self.total_lp_amount;
        let mut amounts = SizedU128Array::from_array(env, [0u128; 3]);

        let d1 = d0 - lp_amount;
        let mut indices = [0, 1, 2];
        // Bubble sort implementation for indices
        for i in 0..indices.len() {
            for j in 0..indices.len() - 1 - i {
                if self.token_balances.get(indices[j]) < self.token_balances.get(indices[j + 1]) {
                    indices.swap(j, j + 1);
                }
            }
        }
        let [more, less, mid] = indices;

        let more_token_amount_sp = self.token_balances.get(more) * lp_amount / d0;
        let mid_token_amount_sp = self.token_balances.get(mid) * lp_amount / d0;
        let y = self.get_y([
            self.token_balances.get(more) - more_token_amount_sp,
            self.token_balances.get(mid) - mid_token_amount_sp,
            d1,
        ])?;
        let less_token_amount_sp = self.token_balances.get(less) - y;

        let mut new_token_balances = self.token_balances.clone();
        let mut fees = SizedU128Array::from_array(env, [0u128; 3]);

        for (index, token_amount_sp) in [
            (more, more_token_amount_sp),
            (mid, mid_token_amount_sp),
            (less, less_token_amount_sp),
        ] {
            let token_amount = self.amount_from_system_precision(token_amount_sp, index);
            let fee = token_amount * self.fee_share_bp / Self::BP;

            let token_amount_sp = self.amount_to_system_precision(token_amount - fee, index);

            fees.set(index, fee);
            amounts.set(index, token_amount_sp);
            new_token_balances.sub(index, token_amount_sp);
        }

        Ok(WithdrawAmount {
            indexes: [more, mid, less],
            fees,
            amounts,
            new_token_balances,
        })
    }
}
