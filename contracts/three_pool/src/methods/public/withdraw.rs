use shared::{Error, Event};
use soroban_sdk::{Address, Env};

use crate::{
    events::{RewardsClaimed, Withdraw},
    methods::internal::pool::Pool,
    storage::user_deposit::UserDeposit,
};

pub fn withdraw<P: Pool>(env: Env, sender: Address, lp_amount: u128) -> Result<(), Error> {
    sender.require_auth();
    let mut pool = P::get(&env)?;
    let mut user_deposit = UserDeposit::get(&env, sender.clone());

    let (withdraw_amount, rewards) =
        pool.withdraw(&env, sender.clone(), &mut user_deposit, lp_amount)?;

    pool.save(&env);
    user_deposit.save(&env, sender.clone());

    Withdraw {
        user: sender.clone(),
        lp_amount,
        amounts: (
            withdraw_amount.amounts.get(0),
            withdraw_amount.amounts.get(1),
            withdraw_amount.amounts.get(2),
        ),
        fees: (
            withdraw_amount.fees.get(0),
            withdraw_amount.fees.get(1),
            withdraw_amount.fees.get(2),
        ),
    }
    .publish(&env);

    if !rewards.iter().sum::<u128>() == 0 {
        RewardsClaimed {
            user: sender,
            rewards: (rewards.get(0), rewards.get(1), rewards.get(2)),
        }
        .publish(&env);
    }

    Ok(())
}
