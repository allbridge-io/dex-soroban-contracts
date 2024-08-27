use std::ops::Index;

use super::{TestingEnv, User};

pub struct UserBalance<const N: usize> {
    pub balances: [u128; N],
    pub lp_amount: u128,
}

pub trait Snapshot<const N: usize>:
    Index<String, Output = u128> + Index<&'static str, Output = u128> + Clone
{
    type TestingEnv: TestingEnv<N>;

    fn get_user_balances(&self, user: &User) -> UserBalance<N>;
    fn take(testing_env: &impl TestingEnv<N>) -> Self;
    fn print_change_with(&self, other: &Self, title: &str);

    fn get_user_balances_sum(&self, user: &User) -> u128 {
        let user_balance = self.get_user_balances(user);
        user_balance.balances.iter().sum::<u128>()
    }

    fn print_changes(
        &self,
        balances: &Vec<(impl ToString, impl ToString, Option<u32>)>,
        other: &Self,
    ) {
        for (title, value_key, use_float_diff) in balances.iter() {
            let title = title.to_string();
            let (before, after) = (self[value_key.to_string()], other[value_key.to_string()]);

            match use_float_diff {
                Some(decimals) => {
                    let (balance_diff, diff) =
                        super::format_diff_with_float_diff(before, after, *decimals);
                    if diff.is_empty() {
                        println!("{}: {}", title, &balance_diff);
                    } else {
                        println!("{}: {} ({})", title, &balance_diff, &diff);
                    }
                }
                None => println!("{}: {}", title, super::format_diff(before, after)),
            }
        }
    }
}
