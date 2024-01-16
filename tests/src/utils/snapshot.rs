use crate::{contracts::pool::Pool as PoolInfo, utils::format_diff};
use color_print::cformat;
use core::panic;
// use ethnum::U256;
// use shared::utils::num::sqrt;
use std::{cmp::Ordering, ops::Index};

use super::{signed_int_to_float, TestingEnvironment};

// impl PoolInfo {
//     pub fn get_y(&self, native_x: u128) -> u128 {
//         let a4 = self.a << 2;
//         let ddd = U256::new(self.d * self.d) * self.d;
//         // 4A(D - x) - D
//         let part1 = a4 as i128 * (self.d as i128 - native_x as i128) - self.d as i128;
//         // x * (4AD³ + x(part1²))
//         let part2 = (ddd * a4 + (U256::new((part1 * part1) as u128) * native_x)) * native_x;
//         // (sqrt(part2) + x(part1)) / 8Ax)
//         (sqrt(&part2).as_u128() as i128 + (native_x as i128 * part1)) as u128
//             / ((self.a << 3) * native_x)
//     }
// }

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub pool_info: PoolInfo,

    pub alice_yaro_balance: u128,
    pub alice_yusd_balance: u128,

    pub bob_yaro_balance: u128,
    pub bob_yusd_balance: u128,

    pub pool_yaro_balance: u128,
    pub pool_yusd_balance: u128,

    pub total_lp_amount: u128,
    pub acc_reward_a_per_share_p: u128,
    pub acc_reward_b_per_share_p: u128,
}

impl Index<String> for Snapshot {
    type Output = u128;

    fn index(&self, string: String) -> &Self::Output {
        self.index(string.as_str())
    }
}

impl Index<&str> for Snapshot {
    type Output = u128;

    fn index(&self, string: &str) -> &Self::Output {
        match string {
            "alice_yaro_balance" => &self.alice_yaro_balance,
            "alice_yusd_balance" => &self.alice_yusd_balance,
            "bob_yaro_balance" => &self.bob_yaro_balance,
            "bob_yusd_balance" => &self.bob_yusd_balance,
            "pool_yaro_balance" => &self.pool_yaro_balance,
            "pool_yusd_balance" => &self.pool_yusd_balance,
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
    pub fn take(testing_env: &TestingEnvironment) -> Snapshot {
        let alice_address = testing_env.alice.as_address();
        let bob_address = testing_env.bob.as_address();

        let alice_yaro_balance = testing_env.yaro_token.balance_of(&alice_address);
        let alice_yusd_balance = testing_env.yusd_token.balance_of(&alice_address);

        let bob_yaro_balance = testing_env.yaro_token.balance_of(&bob_address);
        let bob_yusd_balance = testing_env.yusd_token.balance_of(&bob_address);

        let pool_yaro_balance = testing_env.yaro_token.balance_of(&testing_env.pool.id);
        let pool_yusd_balance = testing_env.yusd_token.balance_of(&testing_env.pool.id);

        let pool_info = testing_env.pool.client.get_pool();
        let total_lp_amount = pool_info.total_lp_amount;
        let acc_reward_a_per_share_p = pool_info.acc_reward_a_per_share_p;
        let acc_reward_b_per_share_p = pool_info.acc_reward_b_per_share_p;

        Snapshot {
            pool_info: testing_env.pool.client.get_pool(),
            alice_yaro_balance,
            pool_yaro_balance,
            alice_yusd_balance,
            pool_yusd_balance,
            bob_yaro_balance,
            bob_yusd_balance,
            total_lp_amount,
            acc_reward_a_per_share_p,
            acc_reward_b_per_share_p,
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
                "Pool acc_reward_a_per_share_p",
                self.acc_reward_a_per_share_p,
                other.acc_reward_a_per_share_p,
                false,
            ),
            (
                "Pool acc_reward_b_per_share_p",
                self.acc_reward_b_per_share_p,
                other.acc_reward_b_per_share_p,
                false,
            ),
            (
                "Pool total_lp_amount",
                self.total_lp_amount,
                other.total_lp_amount,
                true,
            ),
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
