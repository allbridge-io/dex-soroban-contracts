use shared::{require, Error};
use soroban_sdk::contracttype;

use crate::storage::{common::Token, double_values::DoubleU128, pool::Pool};

pub struct ReceiveAmount {
    pub token_from_new_balance: u128,
    pub token_to_new_balance: u128,
    pub output: u128,
    pub fee: u128,
}

pub struct WithdrawAmount {
    pub indexes: [usize; 2],
    pub amounts: DoubleU128,
    pub fees: DoubleU128,
    pub new_token_balances: DoubleU128,
}

#[contracttype]
#[derive(Debug)]
pub struct WithdrawAmountView {
    /// system precision
    pub amounts: (u128, u128),
    /// token precision
    pub fees: (u128, u128),
}

impl From<WithdrawAmount> for WithdrawAmountView {
    fn from(value: WithdrawAmount) -> Self {
        Self {
            amounts: value.amounts.data,
            fees: value.fees.data,
        }
    }
}

pub struct DepositAmount {
    pub lp_amount: u128,
    pub new_token_balances: DoubleU128,
}

impl Pool {
    pub fn get_receive_amount(
        &self,
        input: u128,
        token_from: Token,
    ) -> Result<ReceiveAmount, Error> {
        let token_to = token_from.opposite();
        let d0 = self.total_lp_amount;
        let input_sp = self.amount_to_system_precision(input, self.tokens_decimals[token_from]);
        let mut output = 0;

        let token_from_new_balance = self.token_balances[token_from] + input_sp;

        let token_to_new_balance = self.get_y(token_from_new_balance, d0)?;
        if self.token_balances[token_to] > token_to_new_balance {
            output = self.amount_from_system_precision(
                self.token_balances[token_to] - token_to_new_balance,
                self.tokens_decimals[token_to],
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

    pub fn get_send_amount(&self, output: u128, token_to: Token) -> Result<(u128, u128), Error> {
        let token_from = token_to.opposite();
        let d0 = self.total_lp_amount;
        let fee = output * self.fee_share_bp / (Self::BP - self.fee_share_bp);
        let output_with_fee = output + fee;
        let output_sp =
            self.amount_to_system_precision(output_with_fee, self.tokens_decimals[token_to]);
        let mut input = 0;

        let token_to_new_balance = self.token_balances[token_to] - output_sp;

        let token_from_new_amount = self.get_y(token_to_new_balance, d0)?;
        if self.token_balances[token_from] < token_from_new_amount {
            input = self.amount_from_system_precision(
                token_from_new_amount - self.token_balances[token_from],
                self.tokens_decimals[token_from],
            );
        }

        Ok((input, fee))
    }

    pub fn get_withdraw_amount(&self, lp_amount: u128) -> Result<WithdrawAmount, Error> {
        let d0 = self.total_lp_amount;
        let mut amounts = DoubleU128::default();

        let d1 = d0 - lp_amount;
        let (more, less) = if self.token_balances[0] > self.token_balances[1] {
            (0, 1)
        } else {
            (1, 0)
        };

        let more_token_amount = self.token_balances[more] * lp_amount / d0;
        let y = self.get_y(self.token_balances[more] - more_token_amount, d1)?;
        let less_token_amount = self.token_balances[less] - y;

        let mut new_token_balances = self.token_balances.clone();
        let mut fees = DoubleU128::default();

        for (index, token_amount) in [(more, more_token_amount), (less, less_token_amount)] {
            let token_amount =
                self.amount_from_system_precision(token_amount, self.tokens_decimals[index]);
            let fee = token_amount * self.fee_share_bp / Self::BP;

            let token_amount =
                self.amount_to_system_precision(token_amount - fee, self.tokens_decimals[index]);

            fees[index] = fee;
            amounts[index] = token_amount;
            new_token_balances[index] -= token_amount;
        }

        Ok(WithdrawAmount {
            indexes: [more, less],
            fees,
            amounts,
            new_token_balances,
        })
    }

    pub fn get_deposit_amount(&self, amounts: DoubleU128) -> Result<DepositAmount, Error> {
        let d0 = self.total_lp_amount;

        let amounts_sp = DoubleU128::from((
            self.amount_to_system_precision(amounts[0], self.tokens_decimals[0]),
            self.amount_to_system_precision(amounts[1], self.tokens_decimals[1]),
        ));

        let total_amount = amounts_sp.sum();
        require!(total_amount > 0, Error::ZeroAmount);

        let mut new_token_balances = self.token_balances.clone();

        for (index, amount) in amounts.to_array().into_iter().enumerate() {
            if amount == 0 {
                continue;
            }

            new_token_balances[index] += amounts_sp[index];
        }

        let d1 = self.get_d(new_token_balances[0], new_token_balances[1]);

        require!(d1 > d0, Error::Forbidden);
        require!(
            new_token_balances.sum() < Self::MAX_TOKEN_BALANCE,
            Error::PoolOverflow
        );

        let lp_amount = d1 - d0;

        Ok(DepositAmount {
            lp_amount,
            new_token_balances,
        })
    }
}

#[allow(clippy::inconsistent_digit_grouping)]
#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    use shared::{soroban_data::SimpleSorobanData, Error};
    use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env};

    use crate::storage::{common::Token, double_values::DoubleU128, pool::Pool};

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
            let receive_amount = Pool::get(&env)?.get_receive_amount(amount, token_from)?;
            Ok((receive_amount.output, receive_amount.fee))
        }

        pub fn get_send_amount(
            env: Env,
            amount: u128,
            token_to: Token,
        ) -> Result<(u128, u128), Error> {
            Pool::get(&env)?.get_send_amount(amount, token_to)
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

        let input = 10_000_0000000_u128;
        let (output, fee) = pool.get_receive_amount(&input, &Token::A);
        let (calc_input, calc_fee) = pool.get_send_amount(&output, &Token::B);

        println!("input: {}", input);
        println!("output: {}, fee: {}", output, fee);
        println!("calc input: {}, calc fee: {}", calc_input, calc_fee);

        assert_eq!(input, calc_input);
        assert_eq!(fee, calc_fee);
    }
}
