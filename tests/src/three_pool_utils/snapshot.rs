use std::{cmp::Ordering, ops::Index};

use color_print::cformat;

use super::{int_to_float, TestingEnv, User};
use crate::{
    contracts::three_pool::{ThreePool as PoolInfo, UserDeposit},
    three_pool_utils::format_diff,
};

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub pool_info: PoolInfo,

    pub alice_a_balance: u128,
    pub alice_b_balance: u128,
    pub alice_c_balance: u128,

    pub admin_a_balance: u128,
    pub admin_b_balance: u128,
    pub admin_c_balance: u128,

    pub bob_a_balance: u128,
    pub bob_b_balance: u128,
    pub bob_c_balance: u128,

    pub pool_a_balance: u128,
    pub pool_b_balance: u128,
    pub pool_c_balance: u128,

    pub total_lp_amount: u128,
    pub acc_reward_a_per_share_p: u128,
    pub acc_reward_b_per_share_p: u128,
    pub acc_reward_c_per_share_p: u128,

    pub admin_a_fee_rewards: u128,
    pub admin_b_fee_rewards: u128,
    pub admin_c_fee_rewards: u128,

    pub alice_deposit: UserDeposit,
    pub bob_deposit: UserDeposit,

    pub d: u128,
}

impl Index<&String> for Snapshot {
    type Output = u128;

    fn index(&self, string: &String) -> &Self::Output {
        self.index(string.as_str())
    }
}

impl Index<&str> for Snapshot {
    type Output = u128;

    fn index(&self, string: &str) -> &Self::Output {
        match string {
            "alice_b_balance" => &self.alice_b_balance,
            "alice_a_balance" => &self.alice_a_balance,
            "alice_c_balance" => &self.alice_c_balance,
            "alice_deposit_lp" => &self.alice_deposit.lp_amount,

            "bob_a_balance" => &self.bob_a_balance,
            "bob_b_balance" => &self.bob_b_balance,
            "bob_c_balance" => &self.bob_c_balance,
            "bob_deposit_lp" => &self.bob_deposit.lp_amount,

            "pool_a_balance" => &self.pool_a_balance,
            "pool_b_balance" => &self.pool_b_balance,
            "pool_c_balance" => &self.pool_c_balance,

            "admin_a_balance" => &self.admin_a_balance,
            "admin_b_balance" => &self.admin_b_balance,
            "admin_c_balance" => &self.admin_c_balance,

            "acc_reward_a_per_share_p" => &self.acc_reward_a_per_share_p,
            "acc_reward_b_per_share_p" => &self.acc_reward_b_per_share_p,
            "acc_reward_c_per_share_p" => &self.acc_reward_c_per_share_p,

            "admin_a_fee_rewards" => &self.admin_a_fee_rewards,
            "admin_b_fee_rewards" => &self.admin_b_fee_rewards,
            "admin_c_fee_rewards" => &self.admin_c_fee_rewards,

            "total_lp_amount" => &self.total_lp_amount,
            "d" => &self.d,

            _ => panic!("BalancesSnapshot: unknown field: {}", string),
        }
    }
}

pub fn format_diff_with_float_diff(a: u128, b: u128, decimals: u32) -> (String, String) {
    let float_diff = int_to_float(b as i128 - a as i128, decimals as i32);

    let float_diff = match b.partial_cmp(&a).unwrap() {
        Ordering::Equal => String::new(),
        Ordering::Greater => cformat!("<bright-green>+{float_diff}</bright-green>"),
        Ordering::Less => cformat!("<bright-red>{float_diff}</bright-red>"),
    };

    (format_diff(a, b), float_diff)
}

impl Snapshot {
    pub fn get_user_balances(&self, user: &User) -> (u128, u128, u128, u128) {
        (
            self[&format!("{}_a_balance", user.tag)],
            self[&format!("{}_b_balance", user.tag)],
            self[&format!("{}_c_balance", user.tag)],
            self[&format!("{}_deposit_lp", user.tag)],
        )
    }

    pub fn get_user_balances_sum(&self, user: &User) -> u128 {
        let (a, b, c, _) = self.get_user_balances(user);
        a + b + c
    }

    pub fn get_users_balances_sum(&self) -> u128 {
        let alice_balances = self.alice_b_balance + self.alice_a_balance;
        let bob_balances = self.bob_b_balance + self.bob_a_balance;
        alice_balances + bob_balances
    }

    pub fn take(testing_env: &TestingEnv) -> Snapshot {
        let alice_address = testing_env.alice.as_address();
        let bob_address = testing_env.bob.as_address();

        let alice_a_balance = testing_env.token_a.balance_of(&alice_address);
        let alice_b_balance = testing_env.token_b.balance_of(&alice_address);
        let alice_c_balance = testing_env.token_c.balance_of(&alice_address);

        let admin_a_balance = testing_env.token_a.balance_of(testing_env.admin.as_ref());
        let admin_b_balance = testing_env.token_b.balance_of(testing_env.admin.as_ref());
        let admin_c_balance = testing_env.token_c.balance_of(testing_env.admin.as_ref());

        let bob_a_balance = testing_env.token_a.balance_of(&bob_address);
        let bob_b_balance = testing_env.token_b.balance_of(&bob_address);
        let bob_c_balance = testing_env.token_c.balance_of(&bob_address);

        let pool_a_balance = testing_env.token_a.balance_of(&testing_env.pool.id);
        let pool_b_balance = testing_env.token_b.balance_of(&testing_env.pool.id);
        let pool_c_balance = testing_env.token_c.balance_of(&testing_env.pool.id);

        let pool_info = testing_env.pool.client.get_pool();
        let d = testing_env.pool.client.get_d();
        let total_lp_amount = pool_info.total_lp_amount;

        let acc_reward_a_per_share_p = pool_info.acc_rewards_per_share_p.data.get_unchecked(0);
        let acc_reward_b_per_share_p = pool_info.acc_rewards_per_share_p.data.get_unchecked(1);
        let acc_reward_c_per_share_p = pool_info.acc_rewards_per_share_p.data.get_unchecked(2);

        let admin_a_fee_rewards = pool_info.admin_fee_amount.data.get_unchecked(0);
        let admin_b_fee_rewards = pool_info.admin_fee_amount.data.get_unchecked(1);
        let admin_c_fee_rewards = pool_info.admin_fee_amount.data.get_unchecked(2);

        let alice_deposit = testing_env.pool.client.get_user_deposit(&alice_address);
        let bob_deposit = testing_env.pool.client.get_user_deposit(&bob_address);

        Snapshot {
            d,
            pool_info,
            admin_a_balance,
            admin_b_balance,
            admin_c_balance,
            alice_a_balance,
            alice_b_balance,
            alice_c_balance,
            pool_a_balance,
            pool_b_balance,
            pool_c_balance,
            bob_a_balance,
            bob_b_balance,
            bob_c_balance,
            total_lp_amount,
            acc_reward_a_per_share_p,
            acc_reward_b_per_share_p,
            acc_reward_c_per_share_p,
            admin_a_fee_rewards,
            admin_b_fee_rewards,
            admin_c_fee_rewards,
            alice_deposit,
            bob_deposit,
        }
    }

    pub fn print_change_with(&self, other: &Snapshot, title: &str) {
        println!("----------------------| {title} |----------------------");

        let balances = [
            ("Alice a balance change", "alice_a_balance", Some(7)),
            ("Alice b balance change", "alice_b_balance", Some(7)),
            ("Alice c balance change", "alice_c_balance", Some(7)),
            ("Alice lp change", "alice_deposit_lp", Some(3)),
            ("Bob a balance change", "bob_a_balance", Some(7)),
            ("Bob b balance change", "bob_b_balance", Some(7)),
            ("Bob c balance change", "bob_c_balance", Some(7)),
            ("Bob lp change", "bob_deposit_lp", Some(3)),
            ("Pool a balance change", "pool_a_balance", Some(7)),
            ("Pool b balance change", "pool_b_balance", Some(7)),
            ("Pool c balance change", "pool_c_balance", Some(7)),
            ("Admin a balance change", "admin_a_balance", Some(7)),
            ("Admin b balance change", "admin_b_balance", Some(7)),
            ("Admin c balance change", "admin_c_balance", Some(7)),
            (
                "Pool acc reward a per share p",
                "acc_reward_a_per_share_p",
                None,
            ),
            (
                "Pool acc reward b per share p",
                "acc_reward_b_per_share_p",
                None,
            ),
            (
                "Pool acc reward c per share p",
                "acc_reward_c_per_share_p",
                None,
            ),
            ("Pool admin a fee rewards", "admin_a_fee_rewards", Some(7)),
            ("Pool admin b fee rewards", "admin_b_fee_rewards", Some(7)),
            ("Pool admin c fee rewards", "admin_c_fee_rewards", Some(7)),
            ("Pool total lp amount", "total_lp_amount", Some(3)),
            ("Pool d", "d", Some(3)),
        ];

        for (title, value_key, use_float_diff) in balances {
            let (before, after) = (self[value_key], other[value_key]);

            match use_float_diff {
                Some(decimals) => {
                    let (balance_diff, diff) = format_diff_with_float_diff(before, after, decimals);
                    if diff.is_empty() {
                        println!("{}: {}", title, &balance_diff);
                    } else {
                        println!("{}: {} ({})", title, &balance_diff, &diff);
                    }
                }
                None => println!("{}: {}", title, format_diff(before, after)),
            }
        }
    }
}
