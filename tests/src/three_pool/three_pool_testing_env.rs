use soroban_sdk::{Address, Env};

use crate::{
    contracts::three_pool::{Deposit, RewardsClaimed, Swapped, ThreeToken, Withdraw},
    contracts_wrappers::{EventAsserts, PoolFactory, TestingEnvConfig, ThreePool, Token, User},
    utils::{assert_rel_eq, float_to_uint, float_to_uint_sp, floats_to_uint, get_latest_event},
};

use crate::utils::percentage_to_bp;

use super::ThreePoolSnapshot;

pub struct ThreePoolTestingEnv {
    pub env: Env,
    pub event_asserts: EventAsserts<3>,

    pub admin: User,
    pub native_token: Token<ThreeToken>,

    pub alice: User,
    pub bob: User,

    pub token_a: Token<ThreeToken>,
    pub token_b: Token<ThreeToken>,
    pub token_c: Token<ThreeToken>,

    pub pool: ThreePool,
    pub factory: PoolFactory,
}

impl Default for ThreePoolTestingEnv {
    fn default() -> Self {
        Self::create(TestingEnvConfig::default())
    }
}

impl ThreePoolTestingEnv {
    pub fn create(config: TestingEnvConfig) -> ThreePoolTestingEnv {
        let env = Env::default();

        env.mock_all_auths();
        env.budget().reset_limits(u64::MAX, u64::MAX);

        let admin = User::generate(&env, "admin");
        let native_token = Token::create(&env, admin.as_ref(), ThreeToken::A, "native");
        let alice = User::generate(&env, "alice");
        let bob = User::generate(&env, "bob");

        let factory = PoolFactory::create(&env, admin.as_ref());

        native_token.default_airdrop(&alice);
        native_token.default_airdrop(&bob);

        let (token_a, token_b, token_c) =
            ThreePoolTestingEnv::generate_tokens(&env, admin.as_ref());
        let pool = ThreePoolTestingEnv::create_pool(
            &env,
            &factory,
            &admin,
            &token_a,
            &token_b,
            &token_c,
            config.pool_fee_share_percentage,
            config.pool_admin_fee_percentage,
            config.admin_init_deposit,
        );

        token_a.default_airdrop(&admin);
        token_b.default_airdrop(&admin);
        token_c.default_airdrop(&admin);

        token_a.default_airdrop(&alice);
        token_b.default_airdrop(&alice);
        token_c.default_airdrop(&alice);

        token_a.default_airdrop(&bob);
        token_b.default_airdrop(&bob);
        token_c.default_airdrop(&bob);

        ThreePoolTestingEnv {
            event_asserts: EventAsserts(env.clone()),
            env,

            admin,
            native_token,

            alice,
            bob,

            token_b,
            token_a,
            token_c,
            pool,
            factory,
        }
    }

    pub fn clear_mock_auth(&self) -> &Self {
        self.env.mock_auths(&[]);
        self
    }

    pub fn get_token(&self, pool_token: ThreeToken) -> &Token<ThreeToken> {
        match pool_token {
            ThreeToken::A => &self.token_a,
            ThreeToken::B => &self.token_b,
            ThreeToken::C => &self.token_c,
        }
    }

    pub fn generate_tokens(
        env: &Env,
        admin: &Address,
    ) -> (Token<ThreeToken>, Token<ThreeToken>, Token<ThreeToken>) {
        let token_a = Token::create(env, admin, ThreeToken::A, "a");
        let token_b = Token::create(env, admin, ThreeToken::B, "b");
        let token_c = Token::create(env, admin, ThreeToken::C, "c");

        (token_a, token_b, token_c)
    }

    fn create_pool(
        env: &Env,
        factory: &PoolFactory,
        admin: &User,
        token_a: &Token<ThreeToken>,
        token_b: &Token<ThreeToken>,
        token_c: &Token<ThreeToken>,
        fee_share_percentage: f64,
        admin_fee_percentage: f64,
        admin_init_deposit: f64,
    ) -> ThreePool {
        let fee_share_bp = percentage_to_bp(fee_share_percentage);
        let admin_fee_bp = percentage_to_bp(admin_fee_percentage);
        let a = 20;
        let pool = factory.create_pool(
            admin.as_ref(),
            a,
            [token_a.id.clone(), token_b.id.clone(), token_c.id.clone()],
            fee_share_bp,
            admin_fee_bp,
        );

        let pool = ThreePool::new(env, pool);

        pool.assert_initialization(a, fee_share_bp, admin_fee_bp);

        token_a.airdrop(admin, admin_init_deposit * 2.0);
        token_b.airdrop(admin, admin_init_deposit * 2.0);
        token_c.airdrop(admin, admin_init_deposit * 2.0);

        if admin_init_deposit > 0.0 {
            pool.deposit(
                admin,
                [admin_init_deposit, admin_init_deposit, admin_init_deposit],
                0.0,
            );
        }

        pool
    }

    pub fn assert_deposit(
        &self,
        snapshot_before: ThreePoolSnapshot,
        snapshot_after: ThreePoolSnapshot,
        user: &User,
        expected_deposits: [f64; 3],
        expected_rewards: [f64; 3],
        expected_lp_amount: f64,
    ) {
        self.event_asserts
            .assert_deposit_event(user, expected_lp_amount, expected_deposits);
        self.assert_deposit_without_event(
            snapshot_before,
            snapshot_after,
            user,
            expected_deposits,
            expected_rewards,
            expected_lp_amount,
        );
    }

    pub fn assert_deposit_without_event(
        &self,
        snapshot_before: ThreePoolSnapshot,
        snapshot_after: ThreePoolSnapshot,
        user: &User,
        deposits: [f64; 3],
        expected_rewards: [f64; 3],
        expected_lp_amount: f64,
    ) {
        self.pool.assert_total_lp_less_or_equal_d();

        let (user_a_before, user_b_before, user_c_before, user_lp_amount_before) =
            snapshot_before.get_user_balances(user);
        let (user_a_after, user_b_after, user_c_after, user_lp_amount_after) =
            snapshot_after.get_user_balances(user);

        let deposits = floats_to_uint(deposits, 7);
        let expected_rewards = floats_to_uint(expected_rewards, 7);

        let expected_lp_amount = float_to_uint_sp(expected_lp_amount);

        let user_lp_diff = user_lp_amount_after - user_lp_amount_before;
        let user_a_diff = deposits[0] - (user_a_before - user_a_after);
        let user_b_diff = deposits[1] - (user_b_before - user_b_after);
        let user_c_diff = deposits[2] - (user_c_before - user_c_after);

        let pool_a_diff =
            deposits[0] - (snapshot_after.pool_a_balance - snapshot_before.pool_a_balance);
        let pool_b_diff =
            deposits[1] - (snapshot_after.pool_b_balance - snapshot_before.pool_b_balance);
        let pool_c_diff =
            deposits[2] - (snapshot_after.pool_c_balance - snapshot_before.pool_c_balance);

        assert!(snapshot_before.total_lp_amount < snapshot_after.total_lp_amount);
        assert_eq!(
            snapshot_after.total_lp_amount - snapshot_before.total_lp_amount,
            user_lp_diff
        );
        assert!(snapshot_before.d < snapshot_after.d);
        assert_eq!(user_lp_diff, expected_lp_amount);

        assert_eq!(user_a_diff, expected_rewards[0]);
        assert_eq!(pool_a_diff, expected_rewards[0]);

        assert_eq!(user_b_diff, expected_rewards[1]);
        assert_eq!(pool_b_diff, expected_rewards[1]);

        assert_eq!(user_c_diff, expected_rewards[2]);
        assert_eq!(pool_c_diff, expected_rewards[2]);
    }

    pub fn assert_withdraw(
        &self,
        snapshot_before: ThreePoolSnapshot,
        snapshot_after: ThreePoolSnapshot,
        user: &User,
        expected_amounts: [f64; 3],
        expected_fees: [f64; 3],
        expected_rewards: [f64; 3],
        expected_user_withdraw_lp_diff: f64,
        expected_admin_fees: [f64; 3],
    ) {
        self.pool.assert_total_lp_less_or_equal_d();
        self.event_asserts.assert_withdraw_event(
            user,
            expected_user_withdraw_lp_diff,
            expected_amounts,
            expected_fees,
        );

        let (user_a_before, user_b_before, user_c_before, user_lp_amount_before) =
            snapshot_before.get_user_balances(user);
        let (user_a_after, user_b_after, user_c_after, user_lp_amount_after) =
            snapshot_after.get_user_balances(user);

        let user_a_diff = user_a_after - user_a_before;
        let user_b_diff = user_b_after - user_b_before;
        let user_c_diff = user_c_after - user_c_before;
        let user_lp_diff = user_lp_amount_before - user_lp_amount_after;

        let [expected_a_diff, expected_b_diff, expected_c_diff] = floats_to_uint(
            [
                expected_amounts[0] + expected_rewards[0],
                expected_amounts[1] + expected_rewards[1],
                expected_amounts[2] + expected_rewards[2],
            ],
            7,
        );

        let [expected_a_admin_fee, expected_b_admin_fee, expected_c_admin_fee] =
            floats_to_uint(expected_admin_fees, 7);

        let pool_b_diff = snapshot_before.pool_b_balance - snapshot_after.pool_b_balance;
        let pool_a_diff = snapshot_before.pool_a_balance - snapshot_after.pool_a_balance;
        let pool_c_diff = snapshot_before.pool_c_balance - snapshot_after.pool_c_balance;
        let expected_user_withdraw_lp_amount = float_to_uint_sp(expected_user_withdraw_lp_diff);

        let admin_a_fee_diff =
            snapshot_after.admin_a_fee_rewards - snapshot_before.admin_a_fee_rewards;
        let admin_b_fee_diff =
            snapshot_after.admin_b_fee_rewards - snapshot_before.admin_b_fee_rewards;
        let admin_c_fee_diff =
            snapshot_after.admin_c_fee_rewards - snapshot_before.admin_c_fee_rewards;

        assert_eq!(admin_a_fee_diff, expected_a_admin_fee);
        assert_eq!(admin_b_fee_diff, expected_b_admin_fee);
        assert_eq!(admin_c_fee_diff, expected_c_admin_fee);

        assert!(snapshot_before.total_lp_amount > snapshot_after.total_lp_amount);
        let pool_lp_amount_diff = snapshot_before.total_lp_amount - snapshot_after.total_lp_amount;

        assert!(snapshot_before.d > snapshot_after.d);
        assert_eq!(user_lp_diff, pool_lp_amount_diff);
        assert_eq!(user_lp_diff, expected_user_withdraw_lp_amount);
        assert_eq!(pool_lp_amount_diff, expected_user_withdraw_lp_amount);

        if expected_fees.iter().sum::<f64>() != 0.0 {
            assert!(
                snapshot_before.acc_reward_a_per_share_p < snapshot_after.acc_reward_a_per_share_p
            );
            assert!(
                snapshot_before.acc_reward_b_per_share_p < snapshot_after.acc_reward_b_per_share_p
            );
            assert!(
                snapshot_before.acc_reward_c_per_share_p < snapshot_after.acc_reward_c_per_share_p
            );
        }

        assert_rel_eq(user_a_diff, expected_a_diff, 1);
        assert_rel_eq(user_b_diff, expected_b_diff, 1);
        assert_rel_eq(user_c_diff, expected_c_diff, 1);
        assert_rel_eq(pool_a_diff, expected_a_diff, 1);
        assert_rel_eq(pool_b_diff, expected_b_diff, 1);
        assert_rel_eq(pool_c_diff, expected_c_diff, 1);
    }

    pub fn assert_claim(
        &self,
        snapshot_before: ThreePoolSnapshot,
        snapshot_after: ThreePoolSnapshot,
        user: &User,
        rewards: [f64; 3],
    ) {
        self.pool.assert_total_lp_less_or_equal_d();
        if rewards.iter().sum::<f64>() != 0.0 {
            self.event_asserts
                .assert_claimed_reward_event(user, rewards);
        }

        let (user_a_before, user_b_before, user_c_before, _) =
            snapshot_before.get_user_balances(user);
        let (user_a_after, user_b_after, user_c_after, _) = snapshot_after.get_user_balances(user);

        let user_diffs = [
            user_a_after - user_a_before,
            user_b_after - user_b_before,
            user_c_after - user_c_before,
        ];

        let pool_diffs = [
            snapshot_before.pool_a_balance - snapshot_after.pool_a_balance,
            snapshot_before.pool_b_balance - snapshot_after.pool_b_balance,
            snapshot_before.pool_c_balance - snapshot_after.pool_c_balance,
        ];

        let rewards = floats_to_uint(rewards, 7);

        for i in 0..3 {
            assert_eq!(user_diffs[i], rewards[i]);
            assert_eq!(pool_diffs[i], rewards[i]);
        }
    }

    pub fn assert_claim_admin_fee(
        snapshot_before: ThreePoolSnapshot,
        snapshot_after: ThreePoolSnapshot,
        rewards: [f64; 3],
    ) {
        let [a_reward, b_reward, c_reward] = floats_to_uint(rewards, 7);

        let admin_b_diff = snapshot_after.admin_b_balance - snapshot_before.admin_b_balance;
        let admin_a_diff = snapshot_after.admin_a_balance - snapshot_before.admin_a_balance;
        let admin_c_diff = snapshot_after.admin_c_balance - snapshot_before.admin_c_balance;

        let pool_a_diff = snapshot_before.pool_a_balance - snapshot_after.pool_a_balance;
        let pool_b_diff = snapshot_before.pool_b_balance - snapshot_after.pool_b_balance;
        let pool_c_diff = snapshot_before.pool_c_balance - snapshot_after.pool_c_balance;

        assert_rel_eq(admin_a_diff, a_reward, 2);
        assert_rel_eq(admin_b_diff, b_reward, 2);
        assert_rel_eq(admin_c_diff, c_reward, 2);
        assert_rel_eq(pool_a_diff, a_reward, 2);
        assert_rel_eq(pool_b_diff, b_reward, 2);
        assert_rel_eq(pool_c_diff, c_reward, 2);
    }

    pub fn assert_swap(
        &self,
        snapshot_before: ThreePoolSnapshot,
        snapshot_after: ThreePoolSnapshot,
        sender: &User,
        recipient: &User,
        token_from: &Token<ThreeToken>,
        token_to: &Token<ThreeToken>,
        amount: f64,
        expected_receive_amount: f64,
        expected_fee: f64,
    ) {
        self.pool.assert_total_lp_less_or_equal_d();

        self.event_asserts.assert_swapped_event(
            sender,
            recipient,
            token_from,
            token_to,
            amount,
            expected_receive_amount,
            expected_fee,
        );

        let sender_tag = sender.tag;
        let recipient_tag = recipient.tag;

        let (from_token_tag, to_token_tag) = (token_from.tag.clone(), token_to.tag.clone());

        let sender_balance_key = format!("{sender_tag}_{from_token_tag}_balance");
        let recipient_balance_key = format!("{recipient_tag}_{to_token_tag}_balance");
        let pool_from_balance_key = format!("pool_{from_token_tag}_balance");
        let pool_to_balance_key = format!("pool_{to_token_tag}_balance");
        let acc_reward_token_to_per_share_p_key = format!("acc_reward_{to_token_tag}_per_share_p");

        let expected_receive_amount = float_to_uint(expected_receive_amount, 7);
        let _expected_fee = float_to_uint(expected_fee, 7);
        let amount = float_to_uint(amount, 7);

        let sender_from_token_diff =
            snapshot_before[&sender_balance_key] - snapshot_after[&sender_balance_key];

        let recipient_to_token_diff =
            snapshot_after[&recipient_balance_key] - snapshot_before[&recipient_balance_key];

        let pool_from_token_diff =
            snapshot_after[&pool_from_balance_key] - snapshot_before[&pool_from_balance_key];
        let pool_to_token_diff =
            snapshot_before[&pool_to_balance_key] - snapshot_after[&pool_to_balance_key];

        assert!(
            snapshot_after[&acc_reward_token_to_per_share_p_key]
                > snapshot_before[&acc_reward_token_to_per_share_p_key]
        );

        assert_eq!(recipient_to_token_diff, expected_receive_amount);
        assert_eq!(pool_to_token_diff, expected_receive_amount);

        assert_eq!(pool_to_token_diff, expected_receive_amount);
        assert_eq!(recipient_to_token_diff, expected_receive_amount);

        assert_eq!(sender_from_token_diff, amount);
        assert_eq!(pool_from_token_diff, amount);
    }

    pub fn do_deposit(
        &self,
        user: &User,
        deposit: [f64; 3],
        expected_rewards: [f64; 3],
        expected_lp_amount: f64,
    ) -> (ThreePoolSnapshot, ThreePoolSnapshot) {
        let snapshot_before = ThreePoolSnapshot::take(self);
        self.pool.deposit(user, deposit, 0.0);
        let snapshot_after = ThreePoolSnapshot::take(self);

        let title = format!(
            "Deposit {} a, {} b, expected lp: {expected_lp_amount}",
            deposit[0], deposit[1]
        );
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_deposit(
            snapshot_before.clone(),
            snapshot_after.clone(),
            user,
            deposit,
            expected_rewards,
            expected_lp_amount,
        );

        if expected_rewards.iter().sum::<f64>() != 0.0 {
            self.event_asserts
                .assert_claimed_reward_event(user, expected_rewards);
        }

        (snapshot_before, snapshot_after)
    }

    pub fn do_swap(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        token_from: &Token<ThreeToken>,
        token_to: &Token<ThreeToken>,
        expected_receive_amount: f64,
        expected_fee: f64,
    ) -> (ThreePoolSnapshot, ThreePoolSnapshot) {
        let snapshot_before = ThreePoolSnapshot::take(self);
        self.pool.swap(
            sender,
            recipient,
            amount,
            receive_amount_min,
            token_from,
            token_to,
        );
        let snapshot_after = ThreePoolSnapshot::take(self);

        let title = format!("Swap {amount} a => {expected_receive_amount} b");
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_swap(
            snapshot_before.clone(),
            snapshot_after.clone(),
            sender,
            recipient,
            token_from,
            token_to,
            amount,
            expected_receive_amount,
            expected_fee,
        );

        (snapshot_before, snapshot_after)
    }

    pub fn do_claim(&self, user: &User, expected_rewards: [f64; 3]) {
        let snapshot_before = ThreePoolSnapshot::take(self);
        self.pool.claim_rewards(user);
        let snapshot_after = ThreePoolSnapshot::take(self);

        let title = format!("Claim rewards, expected {:?}", expected_rewards);
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_claim(snapshot_before, snapshot_after, user, expected_rewards);
    }

    pub fn do_claim_admin_fee(&self, expected_rewards: [f64; 3]) {
        let snapshot_before = ThreePoolSnapshot::take(self);
        self.pool.claim_admin_fee();
        let snapshot_after = ThreePoolSnapshot::take(self);

        let title = format!("Claim admin fee, expected {:?}", expected_rewards);
        snapshot_before.print_change_with(&snapshot_after, &title);

        ThreePoolTestingEnv::assert_claim_admin_fee(
            snapshot_before,
            snapshot_after,
            expected_rewards,
        );
    }

    pub fn do_withdraw(
        &self,
        user: &User,
        withdraw_amount: f64,
        expected_withdraw_amounts: [f64; 3],
        expected_fee: [f64; 3],
        expected_rewards: [f64; 3],
        expected_user_lp_diff: f64,
        expected_admin_fee: [f64; 3],
    ) -> (ThreePoolSnapshot, ThreePoolSnapshot) {
        let snapshot_before = ThreePoolSnapshot::take(self);
        self.pool.withdraw(user, withdraw_amount);
        let snapshot_after = ThreePoolSnapshot::take(self);
        snapshot_before.print_change_with(&snapshot_after, "Withdraw");

        if expected_rewards.iter().sum::<f64>() != 0.0 {
            self.event_asserts
                .assert_claimed_reward_event(user, expected_rewards);
        }

        self.assert_withdraw(
            snapshot_before.clone(),
            snapshot_after.clone(),
            user,
            expected_withdraw_amounts,
            expected_fee,
            expected_rewards,
            expected_user_lp_diff,
            expected_admin_fee,
        );

        (snapshot_before, snapshot_after)
    }
}