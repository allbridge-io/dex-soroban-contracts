use std::ops::Index;

use super::TwoPoolTestingEnv;
use crate::utils::{PoolClient, Snapshot, TestingEnv, User, UserBalance, UserDeposit};

#[derive(Debug, Clone)]
pub struct TwoPoolSnapshot {
    pub alice_a_balance: u128,
    pub alice_b_balance: u128,

    pub admin_a_balance: u128,
    pub admin_b_balance: u128,

    pub bob_b_balance: u128,
    pub bob_a_balance: u128,

    pub pool_a_balance: u128,
    pub pool_b_balance: u128,

    pub total_lp_amount: u128,
    pub acc_reward_a_per_share_p: u128,
    pub acc_reward_b_per_share_p: u128,

    pub admin_a_fee_rewards: u128,
    pub admin_b_fee_rewards: u128,

    pub alice_deposit: UserDeposit,
    pub bob_deposit: UserDeposit,

    pub d: u128,
}

impl Index<&String> for TwoPoolSnapshot {
    type Output = u128;

    fn index(&self, string: &String) -> &Self::Output {
        self.index(string.as_str())
    }
}

impl Index<String> for TwoPoolSnapshot {
    type Output = u128;

    fn index(&self, string: String) -> &Self::Output {
        self.index(string.as_str())
    }
}

impl Index<&str> for TwoPoolSnapshot {
    type Output = u128;

    fn index(&self, string: &str) -> &Self::Output {
        match string {
            "alice_a_balance" => &self.alice_a_balance,
            "alice_b_balance" => &self.alice_b_balance,
            "alice_deposit_lp" => &self.alice_deposit.lp_amount,

            "bob_a_balance" => &self.bob_a_balance,
            "bob_b_balance" => &self.bob_b_balance,
            "bob_deposit_lp" => &self.bob_deposit.lp_amount,

            "pool_a_balance" => &self.pool_a_balance,
            "pool_b_balance" => &self.pool_b_balance,

            "admin_a_balance" => &self.admin_a_balance,
            "admin_b_balance" => &self.admin_b_balance,

            "acc_reward_a_per_share_p" => &self.acc_reward_a_per_share_p,
            "acc_reward_b_per_share_p" => &self.acc_reward_b_per_share_p,

            "admin_a_fee_rewards" => &self.admin_a_fee_rewards,
            "admin_b_fee_rewards" => &self.admin_b_fee_rewards,

            "total_lp_amount" => &self.total_lp_amount,
            "d" => &self.d,

            _ => panic!("BalancesSnapshot: unknown field: {}", string),
        }
    }
}

impl Snapshot<2> for TwoPoolSnapshot {
    type TestingEnv = TwoPoolTestingEnv;

    fn get_user_balances(&self, user: &User) -> UserBalance<2> {
        UserBalance {
            balances: [
                self[&format!("{}_a_balance", user.tag)],
                self[&format!("{}_b_balance", user.tag)],
            ],
            lp_amount: self[&format!("{}_deposit_lp", user.tag)],
        }
    }

    fn take(testing_env: &impl TestingEnv<2>) -> TwoPoolSnapshot {
        let (alice, bob, admin) = testing_env.users();
        let [token_a, token_b] = testing_env.tokens();

        let alice_address = alice.as_address();
        let bob_address = bob.as_address();

        let alice_b_balance = token_b.balance_of(&alice_address);
        let alice_a_balance = token_a.balance_of(&alice_address);

        let admin_b_balance = token_b.balance_of(admin.as_ref());
        let admin_a_balance = token_a.balance_of(admin.as_ref());

        let bob_b_balance = token_b.balance_of(&bob_address);
        let bob_a_balance = token_a.balance_of(&bob_address);

        let pool_info = testing_env.pool_client().pool_info();

        let pool_b_balance = token_b.balance_of(&pool_info.id);
        let pool_a_balance = token_a.balance_of(&pool_info.id);

        let d = pool_info.d;
        let total_lp_amount = pool_info.total_lp_amount;

        let acc_reward_a_per_share_p = pool_info.acc_rewards_per_share_p.get_unchecked(0);
        let acc_reward_b_per_share_p = pool_info.acc_rewards_per_share_p.get_unchecked(1);

        let admin_a_fee_rewards = pool_info.admin_fee_amount.get_unchecked(0);
        let admin_b_fee_rewards = pool_info.admin_fee_amount.get_unchecked(1);

        let alice_deposit = testing_env
            .pool_client()
            .user_deposit_by_address(&alice_address);
        let bob_deposit = testing_env
            .pool_client()
            .user_deposit_by_address(&bob_address);

        TwoPoolSnapshot {
            d,
            admin_b_balance,
            admin_a_balance,
            alice_b_balance,
            pool_b_balance,
            alice_a_balance,
            pool_a_balance,
            bob_b_balance,
            bob_a_balance,
            total_lp_amount,
            acc_reward_a_per_share_p,
            acc_reward_b_per_share_p,
            admin_a_fee_rewards,
            admin_b_fee_rewards,
            alice_deposit,
            bob_deposit,
        }
    }

    fn print_change_with(&self, other: &TwoPoolSnapshot, title: &str) {
        println!("----------------------| {title} |----------------------");

        let balances = vec![
            ("Alice b balance change", "alice_b_balance", Some(7)),
            ("Alice a balance change", "alice_a_balance", Some(7)),
            ("Alice lp change", "alice_deposit_lp", Some(3)),
            ("Bob b balance change", "bob_b_balance", Some(7)),
            ("Bob a balance change", "bob_a_balance", Some(7)),
            ("Bob lp change", "bob_deposit_lp", Some(3)),
            ("Pool b balance change", "pool_b_balance", Some(7)),
            ("Pool a balance change", "pool_a_balance", Some(7)),
            ("Admin b balance change", "admin_b_balance", Some(7)),
            ("Admin a balance change", "admin_a_balance", Some(7)),
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
            ("Pool admin a fee rewards", "admin_a_fee_rewards", Some(7)),
            ("Pool admin b fee rewards", "admin_b_fee_rewards", Some(7)),
            ("Pool total lp amount", "total_lp_amount", Some(3)),
            ("Pool d", "d", Some(3)),
        ];

        self.print_changes(&balances, other);
    }
}

impl TwoPoolSnapshot {
    pub fn get_users_balances_sum(&self) -> u128 {
        let alice_balances = self.alice_b_balance + self.alice_a_balance;
        let bob_balances = self.bob_b_balance + self.bob_a_balance;
        alice_balances + bob_balances
    }
}
