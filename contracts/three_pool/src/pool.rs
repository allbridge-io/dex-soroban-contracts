use ethnum::I256;
use shared::{utils::num::*, Error};
use soroban_sdk::{Address, Env};

use generic_pool::prelude::*;

use crate::POOL_SIZE;

use super::token::ThreeToken;

generate_pool_structure!(ThreePool);

impl PoolMath<POOL_SIZE> for ThreePool {
    fn get_d(&self, values: [u128; POOL_SIZE]) -> Result<u128, Error> {
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

    fn get_y(&self, values: [u128; POOL_SIZE]) -> Result<u128, Error> {
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

impl Pool<POOL_SIZE> for ThreePool {
    type Token = ThreeToken;

    fn from_init_params(
        env: &Env,
        a: u128,
        tokens: [Address; POOL_SIZE],
        decimals: [u32; POOL_SIZE],
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
        let token_third = token_from.third(token_to);
        calc_receive_amount(
            self,
            input,
            token_from,
            token_to,
            |pool, token_from_new_balance, d0| {
                pool.get_y([
                    token_from_new_balance,
                    self.token_balances.get(token_third),
                    d0,
                ])
            },
        )
    }

    fn get_send_amount(
        &self,
        output: u128,
        token_from: Self::Token,
        token_to: Self::Token,
    ) -> Result<(u128, u128), Error> {
        let token_third = token_from.third(token_to);
        calc_send_amount(
            self,
            output,
            token_from,
            token_to,
            |pool, token_to_new_balance, d0| {
                pool.get_y([
                    token_to_new_balance,
                    pool.token_balances.get(token_third),
                    d0,
                ])
            },
        )
    }

    fn get_withdraw_amount(&self, env: &Env, lp_amount: u128) -> Result<WithdrawAmount<3>, Error> {
        calc_withdraw_amount(self, env, lp_amount, &[0, 2])
    }
}
