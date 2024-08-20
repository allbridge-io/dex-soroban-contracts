use shared::{require, Error};
use soroban_sdk::{contracttype, Env};

use crate::{
    common::{DepositAmount, Pool, PoolView, ReceiveAmount, WithdrawAmount},
    storage::sized_array::SizedU128Array,
};

use super::{token::Token, two_pool::TwoPool, utils::get_double_tuple_from_sized_u128_array};

#[contracttype]
#[derive(Debug)]
pub struct WithdrawAmountView {
    /// system precision
    pub amounts: (u128, u128),
    /// token precision
    pub fees: (u128, u128),
}

impl<const N: usize> From<WithdrawAmount<N>> for WithdrawAmountView {
    fn from(value: WithdrawAmount<N>) -> Self {
        Self {
            amounts: get_double_tuple_from_sized_u128_array(value.amounts),
            fees: get_double_tuple_from_sized_u128_array(value.fees),
        }
    }
}

impl PoolView<2, Token> for TwoPool {
    fn get_receive_amount(
        &self,
        input: u128,
        token_from: Token,
        _token_to: Token,
    ) -> Result<ReceiveAmount, Error> {
        let token_to = token_from.opposite();
        let d0 = self.total_lp_amount;
        let input_sp = self.amount_to_system_precision(input, self.tokens_decimals.get(token_from));
        let mut output = 0;

        let token_from_new_balance = self.token_balances.get(token_from) + input_sp;

        let token_to_new_balance = self.get_y([token_from_new_balance, d0])?;
        if self.token_balances.get(token_to) > token_to_new_balance {
            output = self.amount_from_system_precision(
                self.token_balances.get(token_to) - token_to_new_balance,
                self.tokens_decimals.get(token_to),
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
        _token_from: Token,
        token_to: Token,
    ) -> Result<(u128, u128), Error> {
        let token_from = token_to.opposite();
        let d0 = self.total_lp_amount;
        let fee = output * self.fee_share_bp / (Self::BP - self.fee_share_bp);
        let output_with_fee = output + fee;
        let output_sp =
            self.amount_to_system_precision(output_with_fee, self.tokens_decimals.get(token_to));
        let mut input = 0;

        let token_to_new_balance = self.token_balances.get(token_to) - output_sp;

        let token_from_new_amount = self.get_y([token_to_new_balance, d0])?;
        if self.token_balances.get(token_from) < token_from_new_amount {
            input = self.amount_from_system_precision(
                token_from_new_amount - self.token_balances.get(token_from),
                self.tokens_decimals.get(token_from),
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
            let token_amount =
                self.amount_from_system_precision(token_amount_sp, self.tokens_decimals.get(index));
            let fee = token_amount * self.fee_share_bp / Self::BP;

            let token_amount_sp = self
                .amount_to_system_precision(token_amount - fee, self.tokens_decimals.get(index));

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

    fn get_deposit_amount(
        &self,
        env: &Env,
        amounts: SizedU128Array,
    ) -> Result<DepositAmount, Error> {
        let d0 = self.total_lp_amount;

        let amounts_sp = SizedU128Array::from_array(
            env,
            [
                self.amount_to_system_precision(
                    amounts.get(0usize),
                    self.tokens_decimals.get(0usize),
                ),
                self.amount_to_system_precision(
                    amounts.get(1usize),
                    self.tokens_decimals.get(1usize),
                ),
            ],
        );

        let total_amount_sp = amounts_sp.iter().sum::<u128>();
        require!(total_amount_sp > 0, Error::ZeroAmount);

        let mut new_token_balances_sp = self.token_balances.clone();

        for (index, amount) in amounts.iter().enumerate() {
            if amount == 0 {
                continue;
            }

            new_token_balances_sp.add(index, amounts_sp.get(index));
        }

        let d1 = self.get_d([
            new_token_balances_sp.get(0usize),
            new_token_balances_sp.get(1usize),
        ])?;

        require!(d1 > d0, Error::Forbidden);
        require!(
            new_token_balances_sp.iter().sum::<u128>() < Self::MAX_TOKEN_BALANCE,
            Error::PoolOverflow
        );

        let lp_amount = d1 - d0;

        Ok(DepositAmount {
            lp_amount,
            new_token_balances: new_token_balances_sp.get_inner(),
        })
    }
}

// #[allow(clippy::inconsistent_digit_grouping)]
// #[cfg(test)]
// mod tests {
//     extern crate std;
//     use std::println;

//     use shared::{soroban_data::SimpleSorobanData, Error};
//     use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

//     use crate::storage::{common::Token, double_values::DoubleU128, pool::Pool};

//     #[contract]
//     pub struct TestPool;

//     #[contractimpl]
//     impl TestPool {
//         pub fn init(env: Env) {
//             let token_a = Address::generate(&env);
//             let token_b = Address::generate(&env);
//             Pool::from_init_params(20, [token_a, token_b], [7, 7], 100, 1).save(&env);
//         }

//         pub fn set_balances(env: Env, new_balances: (u128, u128)) -> Result<(), Error> {
//             Pool::update(&env, |pool| {
//                 pool.token_balances = DoubleU128::from(new_balances);
//                 pool.total_lp_amount = pool.get_current_d()?;
//                 Ok(())
//             })
//         }

//         pub fn get_receive_amount(
//             env: Env,
//             amount: u128,
//             token_from: Token,
//         ) -> Result<(u128, u128), Error> {
//             let receive_amount = Pool::get(&env)?.get_receive_amount(amount, token_from)?;
//             Ok((receive_amount.output, receive_amount.fee))
//         }

//         pub fn get_send_amount(
//             env: Env,
//             amount: u128,
//             token_to: Token,
//         ) -> Result<(u128, u128), Error> {
//             Pool::get(&env)?.get_send_amount(amount, token_to)
//         }
//     }

//     #[test]
//     fn test() {
//         let env = Env::default();

//         let test_pool_id = env.register_contract(None, TestPool);
//         let pool = TestPoolClient::new(&env, &test_pool_id);
//         pool.init();
//         pool.set_balances(&(200_000_000, 200_000_000));

//         let input = 10_000_0000000_u128;
//         let (output, fee) = pool.get_receive_amount(&input, &Token::A);
//         let (calc_input, calc_fee) = pool.get_send_amount(&output, &Token::B);

//         println!("input: {}", input);
//         println!("output: {}, fee: {}", output, fee);
//         println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

//         assert_eq!(input, calc_input);
//         assert_eq!(fee, calc_fee);
//     }

//     #[test]
//     fn test_disbalance() {
//         let env = Env::default();

//         let test_pool_id = env.register_contract(None, TestPool);
//         let pool = TestPoolClient::new(&env, &test_pool_id);
//         pool.init();
//         pool.set_balances(&(200_000_000, 500_000_000));

//         let input = 10_000_0000000_u128;
//         let (output, fee) = pool.get_receive_amount(&input, &Token::A);
//         let (calc_input, calc_fee) = pool.get_send_amount(&output, &Token::B);

//         println!("input: {}", input);
//         println!("output: {}, fee: {}", output, fee);
//         println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

//         assert_eq!(input, calc_input);
//         assert_eq!(fee, calc_fee);
//     }
// }
