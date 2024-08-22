use ethnum::U256;
use shared::{
    utils::{num::*, safe_cast},
    Error,
};
use soroban_sdk::{Address, Env};

use generic_pool::prelude::*;

use super::token::TwoToken;
use crate::POOL_SIZE;

generate_pool_structure!(TwoPool);

impl PoolMath<POOL_SIZE> for TwoPool {
    // y = (sqrt(x(4AD³ + x (4A(D - x) - D )²)) + x (4A(D - x) - D ))/8Ax
    fn get_y(&self, values: [u128; POOL_SIZE]) -> Result<u128, Error> {
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

    fn get_d(&self, values: [u128; POOL_SIZE]) -> Result<u128, Error> {
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

impl Pool<POOL_SIZE> for TwoPool {
    type Token = TwoToken;

    fn from_init_params(
        env: &Env,
        a: u128,
        tokens: [Address; POOL_SIZE],
        decimals: [u32; POOL_SIZE],
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
            token_balances: SizedU128Array::default_val::<POOL_SIZE>(env),
            acc_rewards_per_share_p: SizedU128Array::default_val::<POOL_SIZE>(env),
            admin_fee_amount: SizedU128Array::default_val::<POOL_SIZE>(env),
        }
    }

    fn get_receive_amount(
        &self,
        input: u128,
        token_from: Self::Token,
        token_to: Self::Token,
    ) -> Result<ReceiveAmount, Error> {
        calc_receive_amount(
            self,
            input,
            token_from,
            token_to,
            |pool, token_from_new_balance, d0| pool.get_y([token_from_new_balance, d0]),
        )
    }

    fn get_send_amount(
        &self,
        output: u128,
        token_from: Self::Token,
        token_to: Self::Token,
    ) -> Result<(u128, u128), Error> {
        calc_send_amount(
            self,
            output,
            token_from,
            token_to,
            |pool, token_to_new_balance, d0| pool.get_y([token_to_new_balance, d0]),
        )
    }

    fn get_withdraw_amount(&self, env: &Env, lp_amount: u128) -> Result<WithdrawAmount<2>, Error> {
        calc_withdraw_amount(self, env, lp_amount, &[0])
    }
}
