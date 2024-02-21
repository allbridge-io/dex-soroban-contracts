use shared::{require, Error};

use crate::storage::{
    double_values::DoubleU128,
    pool::{Pool, Token},
};

pub struct ReceiveAmount {
    pub token_from_new_balance: u128,
    pub token_to_new_balance: u128,
    pub output: u128,
    pub fee: u128,
}

pub struct WithdrawAmount {
    pub indexes: [usize; 2],
    pub amounts: DoubleU128,
    pub new_token_balances: DoubleU128,
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
        let mut withdraw_amounts = DoubleU128::default();

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

        for (index, token_amount) in [(more, more_token_amount), (less, less_token_amount)] {
            withdraw_amounts[index] = token_amount;
            new_token_balances[index] -= token_amount;
        }

        Ok(WithdrawAmount {
            indexes: [more, less],
            amounts: withdraw_amounts,
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
