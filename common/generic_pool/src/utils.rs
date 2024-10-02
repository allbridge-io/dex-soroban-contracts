use shared::Error;
use soroban_sdk::Env;

use crate::{
    pool::{Pool, ReceiveAmount, WithdrawAmount},
    prelude::SizedU128Array,
};

pub fn calc_send_amount<const N: usize, P: Pool<N>>(
    pool: &P,
    output: u128,
    token_from: P::Token,
    token_to: P::Token,
    compute_y: impl FnOnce(&P, u128, u128) -> Result<u128, Error>,
) -> Result<(u128, u128), Error> {
    let d0 = pool.total_lp_amount();
    let fee = output * pool.fee_share_bp() / (P::BP - pool.fee_share_bp());
    let output_with_fee = output + fee;
    let output_sp = pool.amount_to_system_precision(output_with_fee, token_to);
    let mut input = 0;

    let token_to_new_balance = pool.token_balances().get(token_to) - output_sp;

    let token_from_new_amount = compute_y(pool, token_to_new_balance, d0)?;
    if pool.token_balances().get(token_from) < token_from_new_amount {
        input = pool.amount_from_system_precision(
            token_from_new_amount - pool.token_balances().get(token_from),
            token_from,
        );
    }

    Ok((input, fee))
}

pub fn calc_receive_amount<const N: usize, P: Pool<N>>(
    pool: &P,
    input: u128,
    token_from: P::Token,
    token_to: P::Token,
    compute_y: impl FnOnce(&P, u128, u128) -> Result<u128, Error>,
) -> Result<ReceiveAmount, Error> {
    let d0 = pool.total_lp_amount();
    let input_sp = pool.amount_to_system_precision(input, token_from);
    let mut output = 0;

    let token_from_new_balance = pool.token_balances().get(token_from) + input_sp;

    let token_to_new_balance = compute_y(pool, token_from_new_balance, d0)?;
    if pool.token_balances().get(token_to) > token_to_new_balance {
        output = pool.amount_from_system_precision(
            pool.token_balances().get(token_to) - token_to_new_balance,
            token_to,
        );
    }
    let fee = output * pool.fee_share_bp() / P::BP;

    output -= fee;

    Ok(ReceiveAmount {
        token_from_new_balance,
        token_to_new_balance,
        output,
        fee,
    })
}

pub fn calc_withdraw_amount<const N: usize, P: Pool<N>>(
    pool: &P,
    env: &Env,
    lp_amount: u128,
    y_indexes: &[usize],
) -> Result<WithdrawAmount<N>, Error> {
    let d0 = pool.total_lp_amount();
    let mut amounts = SizedU128Array::default_val::<N>(env);
    let d1 = d0 - lp_amount;
    let indexes = generate_indexes(pool);

    let mut token_amounts_sp = [0u128; N];

    for i in 0..N {
        token_amounts_sp[i] = pool.token_balances().get(indexes[i]) * lp_amount / d0;
    }

    let mut y_args = [d1; N];
    for i in 0..(N - 1) {
        y_args[i] =
            pool.token_balances().get(indexes[y_indexes[i]]) - token_amounts_sp[y_indexes[i]];
    }
    let y = pool.get_y(y_args)?;
    token_amounts_sp[1] = pool.token_balances().get(indexes[1]) - y;

    let mut new_token_balances = pool.token_balances().clone();
    let mut fees = SizedU128Array::default_val::<N>(env);

    for i in 0..N {
        let index = indexes[i];
        let token_amount_sp = token_amounts_sp[i];
        let token_amount = pool.amount_from_system_precision(token_amount_sp, index);
        let fee = token_amount * pool.fee_share_bp() / P::BP;

        let token_amount_sp = pool.amount_to_system_precision(token_amount - fee, index);

        fees.set(index, fee);
        amounts.set(index, token_amount_sp);
        new_token_balances.sub(index, token_amount_sp);
    }

    Ok(WithdrawAmount {
        indexes,
        fees,
        amounts,
        new_token_balances,
    })
}

pub fn generate_indexes<const N: usize, P: Pool<N>>(pool: &P) -> [usize; N] {
    let mut indices: [usize; N] = core::array::from_fn(|i| i);
    // Bubble sort implementation for indices
    for i in 0..indices.len() {
        for j in 0..indices.len() - 1 - i {
            if pool.token_balances().get(indices[j]) < pool.token_balances().get(indices[j + 1]) {
                indices.swap(j, j + 1);
            }
        }
    }

    indices
}

#[macro_export]
macro_rules! generate_pool_structure {
    ($name:ident) => {
        #[soroban_sdk::contracttype]
        #[derive(
            Debug,
            Clone,
            proc_macros::SorobanData,
            proc_macros::SorobanSimpleData,
            proc_macros::SymbolKey,
            proc_macros::Instance,
        )]
        #[proc_macros::extend_ttl_info_instance]
        pub struct $name {
            pub a: u128,

            pub fee_share_bp: u128,
            pub admin_fee_share_bp: u128,
            pub total_lp_amount: u128,

            pub tokens: SizedAddressArray,
            pub tokens_decimals: SizedDecimalsArray,
            pub token_balances: SizedU128Array,
            pub acc_rewards_per_share_p: SizedU128Array,
            pub admin_fee_amount: SizedU128Array,
        }

        impl generic_pool::prelude::PoolStorage for $name {
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
    };
}
