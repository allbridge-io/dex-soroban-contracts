use std::{cmp::Ordering, ops::Index};

use color_print::cformat;

use super::{signed_int_to_float, TestingEnvironment, User};
use crate::{
    contracts::pool::{Pool as PoolInfo, UserDeposit},
    utils::format_diff,
};

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub pool_info: PoolInfo,

    pub alice_yaro_balance: u128,
    pub alice_yusd_balance: u128,

    pub admin_yaro_balance: u128,
    pub admin_yusd_balance: u128,

    pub bob_yaro_balance: u128,
    pub bob_yusd_balance: u128,

    pub pool_yaro_balance: u128,
    pub pool_yusd_balance: u128,

    pub total_lp_amount: u128,
    pub acc_reward_yusd_per_share_p: u128,
    pub acc_reward_yaro_per_share_p: u128,

    pub admin_yaro_fee_rewards: u128,
    pub admin_yusd_fee_rewards: u128,

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
            "alice_yaro_balance" => &self.alice_yaro_balance,
            "alice_yusd_balance" => &self.alice_yusd_balance,
            "alice_deposit_lp" => &self.alice_deposit.lp_amount,

            "bob_yaro_balance" => &self.bob_yaro_balance,
            "bob_yusd_balance" => &self.bob_yusd_balance,
            "bob_deposit_lp" => &self.bob_deposit.lp_amount,

            "pool_yaro_balance" => &self.pool_yaro_balance,
            "pool_yusd_balance" => &self.pool_yusd_balance,

            "acc_reward_yusd_per_share_p" => &self.acc_reward_yusd_per_share_p,
            "acc_reward_yaro_per_share_p" => &self.acc_reward_yaro_per_share_p,

            _ => panic!("BalancesSnapshot: unknown field: {}", string),
        }
    }
}

pub fn format_diff_with_float_diff(a: u128, b: u128) -> (String, String) {
    let float_diff = signed_int_to_float(b as i128 - a as i128);

    let float_diff = match b.partial_cmp(&a).unwrap() {
        Ordering::Equal => String::new(),
        Ordering::Greater => cformat!("<bright-green>+{float_diff}</bright-green>"),
        Ordering::Less => cformat!("<bright-red>{float_diff}</bright-red>"),
    };

    (format_diff(a, b), float_diff)
}

impl Snapshot {
    pub fn get_user_balances(&self, user: &User) -> (u128, u128, u128) {
        (
            self[&format!("{}_yusd_balance", user.tag)],
            self[&format!("{}_yaro_balance", user.tag)],
            self[&format!("{}_deposit_lp", user.tag)],
        )
    }

    pub fn get_user_balances_sum(&self) -> u128 {
        let alice_balances = self.alice_yaro_balance + self.alice_yusd_balance;
        let bob_balances = self.bob_yaro_balance + self.bob_yusd_balance;
        alice_balances + bob_balances
    }

    pub fn take(testing_env: &TestingEnvironment) -> Snapshot {
        let alice_address = testing_env.alice.as_address();
        let bob_address = testing_env.bob.as_address();

        let alice_yaro_balance = testing_env.yaro_token.balance_of(&alice_address);
        let alice_yusd_balance = testing_env.yusd_token.balance_of(&alice_address);

        let admin_yaro_balance = testing_env.yaro_token.balance_of(&testing_env.admin);
        let admin_yusd_balance = testing_env.yusd_token.balance_of(&testing_env.admin);

        let bob_yaro_balance = testing_env.yaro_token.balance_of(&bob_address);
        let bob_yusd_balance = testing_env.yusd_token.balance_of(&bob_address);

        let pool_yaro_balance = testing_env.yaro_token.balance_of(&testing_env.pool.id);
        let pool_yusd_balance = testing_env.yusd_token.balance_of(&testing_env.pool.id);

        let pool_info = testing_env.pool.client.get_pool();
        let d = testing_env.pool.client.get_d();
        let total_lp_amount = pool_info.total_lp_amount;

        let acc_reward_yusd_per_share_p = pool_info.acc_rewards_per_share_p.data.0;
        let acc_reward_yaro_per_share_p = pool_info.acc_rewards_per_share_p.data.1;

        let admin_yusd_fee_rewards = pool_info.admin_fee_amount.data.0;
        let admin_yaro_fee_rewards = pool_info.admin_fee_amount.data.1;

        let alice_deposit = testing_env.pool.client.get_user_deposit(&alice_address);
        let bob_deposit = testing_env.pool.client.get_user_deposit(&bob_address);

        Snapshot {
            d,
            pool_info,
            admin_yaro_balance,
            admin_yusd_balance,
            alice_yaro_balance,
            pool_yaro_balance,
            alice_yusd_balance,
            pool_yusd_balance,
            bob_yaro_balance,
            bob_yusd_balance,
            total_lp_amount,
            acc_reward_yusd_per_share_p,
            acc_reward_yaro_per_share_p,
            admin_yusd_fee_rewards,
            admin_yaro_fee_rewards,
            alice_deposit,
            bob_deposit,
        }
    }

    #[allow(dead_code)]
    pub fn print_change_with(&self, other: &Snapshot, title: Option<&str>) {
        let title = title.unwrap_or("Diff");

        println!("----------------------| {title} |----------------------");

        let balances = [
            (
                "Alice yaro balance change",
                self.alice_yaro_balance,
                other.alice_yaro_balance,
                true,
            ),
            (
                "Alice yusd balance change",
                self.alice_yusd_balance,
                other.alice_yusd_balance,
                true,
            ),
            (
                "Alice lp change",
                self.alice_deposit.lp_amount,
                other.alice_deposit.lp_amount,
                false,
            ),
            (
                "Bob yaro balance change",
                self.bob_yaro_balance,
                other.bob_yaro_balance,
                true,
            ),
            (
                "Bob yusd balance change",
                self.bob_yusd_balance,
                other.bob_yusd_balance,
                true,
            ),
            (
                "Bob lp change",
                self.bob_deposit.lp_amount,
                other.bob_deposit.lp_amount,
                false,
            ),
            (
                "Pool yaro balance change",
                self.pool_yaro_balance,
                other.pool_yaro_balance,
                true,
            ),
            (
                "Pool yusd balance change",
                self.pool_yusd_balance,
                other.pool_yusd_balance,
                true,
            ),
            (
                "Admin yaro balance change",
                self.admin_yaro_balance,
                other.admin_yaro_balance,
                true,
            ),
            (
                "Admin yusd balance change",
                self.admin_yusd_balance,
                other.admin_yusd_balance,
                true,
            ),
            (
                "Pool acc reward yusd per share p",
                self.acc_reward_yusd_per_share_p,
                other.acc_reward_yusd_per_share_p,
                false,
            ),
            (
                "Pool acc reward yaro per share p",
                self.acc_reward_yaro_per_share_p,
                other.acc_reward_yaro_per_share_p,
                false,
            ),
            (
                "Pool admin yusd fee rewards",
                self.admin_yusd_fee_rewards,
                other.admin_yusd_fee_rewards,
                false,
            ),
            (
                "Pool admin yaro fee rewards",
                self.admin_yaro_fee_rewards,
                other.admin_yaro_fee_rewards,
                false,
            ),
            (
                "Pool total lp amount",
                self.total_lp_amount,
                other.total_lp_amount,
                false,
            ),
            ("Pool d", self.d, other.d, false),
        ];

        for (title, a, b, use_float_diff) in balances {
            if !use_float_diff {
                println!("{}: {}", title, format_diff(a, b));
                continue;
            }

            let (balance_diff, diff) = format_diff_with_float_diff(a, b);
            if diff.is_empty() {
                println!("{}: {}", title, &balance_diff);
            } else {
                println!("{}: {} ({})", title, &balance_diff, &diff);
            }
        }
    }
}
