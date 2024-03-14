use soroban_sdk::{Address, Env};

use crate::{
    contracts::pool::{Deposit, Direction, RewardsClaimed, Swapped, Withdraw},
    utils::{assert_rel_eq, float_to_uint, float_to_uint_sp, percentage_to_bp},
};

use super::{get_latest_event, Pool, PoolFactory, Snapshot, Token, User};

#[derive(Debug, Clone)]
pub struct TestingEnvConfig {
    /// default: `0.0`, from 0.0 to 100.0
    pub pool_fee_share_percentage: f64,
    /// default: `0.0`, from 0.0 to 100.0
    pub pool_admin_fee_percentage: f64,
    /// default: `100_000.0`
    pub admin_init_deposit: f64,
}

pub const DOUBLE_ZERO: (f64, f64) = (0.0, 0.0);

impl TestingEnvConfig {
    pub fn with_admin_init_deposit(mut self, admin_init_deposit: f64) -> Self {
        self.admin_init_deposit = admin_init_deposit;
        self
    }

    // from 0.0 to 100.0
    pub fn with_pool_admin_fee(mut self, pool_admin_fee_percentage: f64) -> Self {
        assert!((0.0..100.0).contains(&pool_admin_fee_percentage));

        self.pool_admin_fee_percentage = pool_admin_fee_percentage;
        self
    }

    // from 0.0 to 100.0
    pub fn with_pool_fee_share(mut self, fee_share_percentage: f64) -> Self {
        assert!((0.0..=100.0).contains(&fee_share_percentage));

        self.pool_fee_share_percentage = fee_share_percentage;
        self
    }
}

impl Default for TestingEnvConfig {
    fn default() -> Self {
        TestingEnvConfig {
            pool_fee_share_percentage: 0.0,
            pool_admin_fee_percentage: 0.0,
            admin_init_deposit: 100_000.0,
        }
    }
}

pub struct TestingEnv {
    pub env: Env,
    pub admin: User,
    pub native_token: Token,

    pub alice: User,
    pub bob: User,

    pub yaro_token: Token,
    pub yusd_token: Token,

    pub pool: Pool,
    pub factory: PoolFactory,
}

impl Default for TestingEnv {
    fn default() -> Self {
        Self::create(TestingEnvConfig::default())
    }
}

impl TestingEnv {
    pub fn create(config: TestingEnvConfig) -> TestingEnv {
        let env = Env::default();

        env.mock_all_auths();
        env.budget().reset_limits(u64::MAX, u64::MAX);

        let admin = User::generate(&env, "admin");
        let native_token = Token::create(&env, admin.as_ref());
        let alice = User::generate(&env, "alice");
        let bob = User::generate(&env, "bob");

        let factory = PoolFactory::create(&env, admin.as_ref());

        native_token.default_airdrop(&alice);
        native_token.default_airdrop(&bob);

        let (yusd_token, yaro_token) = TestingEnv::generate_token_pair(&env, admin.as_ref());
        let pool = TestingEnv::create_pool(
            &env,
            &factory,
            &admin,
            &yusd_token,
            &yaro_token,
            config.pool_fee_share_percentage,
            config.pool_admin_fee_percentage,
            config.admin_init_deposit,
        );

        yusd_token.default_airdrop(&admin);
        yaro_token.default_airdrop(&admin);

        yusd_token.default_airdrop(&alice);
        yaro_token.default_airdrop(&alice);

        yusd_token.default_airdrop(&bob);
        yaro_token.default_airdrop(&bob);

        TestingEnv {
            env,

            admin,
            native_token,

            alice,
            bob,

            yaro_token,
            yusd_token,
            pool,
            factory,
        }
    }

    pub fn clear_mock_auth(&self) -> &Self {
        self.env.mock_auths(&[]);
        self
    }

    pub fn generate_token_pair(env: &Env, admin: &Address) -> (Token, Token) {
        let token_a = Token::create(env, admin);
        let token_b = Token::create(env, admin);

        (token_a, token_b)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_pool(
        env: &Env,
        factory: &PoolFactory,
        admin: &User,
        token_a: &Token,
        token_b: &Token,
        fee_share_percentage: f64,
        admin_fee_percentage: f64,
        admin_init_deposit: f64,
    ) -> Pool {
        let fee_share_bp = percentage_to_bp(fee_share_percentage);
        let admin_fee_bp = percentage_to_bp(admin_fee_percentage);
        let a = 20;
        let pool = factory.create_pair(
            admin.as_ref(),
            a,
            &token_a.id,
            &token_b.id,
            fee_share_bp,
            admin_fee_bp,
        );

        let pool = Pool::new(env, pool);

        pool.assert_initialization(a, fee_share_bp, admin_fee_bp);

        token_a.airdrop(admin, admin_init_deposit * 2.0);
        token_b.airdrop(admin, admin_init_deposit * 2.0);

        if admin_init_deposit > 0.0 {
            pool.deposit(admin, (admin_init_deposit, admin_init_deposit), 0.0);
        }

        pool
    }

    pub fn assert_claimed_reward_event(
        &self,
        expected_user: &User,
        (expected_yusd_reward, expected_yaro_reward): (f64, f64),
    ) {
        let rewards_claimed =
            get_latest_event::<RewardsClaimed>(&self.env).expect("Expected RewardsClaimed");

        assert_eq!(rewards_claimed.user, expected_user.as_address());
        assert_rel_eq(
            rewards_claimed.rewards.0,
            float_to_uint(expected_yusd_reward, 7),
            1,
        );
        assert_rel_eq(
            rewards_claimed.rewards.1,
            float_to_uint(expected_yaro_reward, 7),
            1,
        );
    }

    pub fn assert_swapped_event(
        &self,
        sender: &User,
        recipient: &User,
        directin: Direction,
        from_amount: f64,
        expected_to_amount: f64,
        expected_fee: f64,
    ) {
        let swapped = get_latest_event::<Swapped>(&self.env).expect("Expected Swapped");

        let (from_token, to_token) = match directin {
            Direction::A2B => (self.yusd_token.as_address(), self.yaro_token.as_address()),
            Direction::B2A => (self.yaro_token.as_address(), self.yusd_token.as_address()),
        };

        assert_eq!(swapped.sender, sender.as_address());
        assert_eq!(swapped.recipient, recipient.as_address());

        assert_eq!(swapped.from_amount, float_to_uint(from_amount, 7));
        assert_eq!(swapped.to_amount, float_to_uint(expected_to_amount, 7));
        assert_rel_eq(swapped.fee, float_to_uint(expected_fee, 7), 1);

        assert_eq!(swapped.from_token, from_token);
        assert_eq!(swapped.to_token, to_token);
    }

    pub fn assert_withdraw_event(
        &self,
        expected_user: &User,
        lp_amount: f64,
        (yusd_amount, yaro_amount): (f64, f64),
        (yusd_fee, yaro_fee): (f64, f64),
    ) {
        let withdraw = get_latest_event::<Withdraw>(&self.env).expect("Expected Withdraw");

        assert_eq!(withdraw.user, expected_user.as_address());
        assert_eq!(withdraw.lp_amount, float_to_uint_sp(lp_amount));

        assert_rel_eq(withdraw.amounts.0, float_to_uint_sp(yusd_amount), 1);
        assert_rel_eq(withdraw.amounts.1, float_to_uint_sp(yaro_amount), 1);

        assert_rel_eq(withdraw.fees.0, float_to_uint(yusd_fee, 7), 1);
        assert_rel_eq(withdraw.fees.1, float_to_uint(yaro_fee, 7), 1);
    }

    pub fn assert_deposit_event(
        &self,
        expected_user: &User,
        expected_lp_amount: f64,
        (yusd_deposit, yaro_deposit): (f64, f64),
    ) {
        let deposit = get_latest_event::<Deposit>(&self.env).expect("Expected Deposit");

        assert_eq!(deposit.user, expected_user.as_address());
        assert_eq!(deposit.amounts.0, float_to_uint(yusd_deposit, 7));
        assert_eq!(deposit.amounts.1, float_to_uint(yaro_deposit, 7));
        assert_eq!(float_to_uint_sp(expected_lp_amount), deposit.lp_amount);
    }

    pub fn assert_deposit(
        &self,
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        expected_deposits: (f64, f64),
        expected_rewards: (f64, f64),
        expected_lp_amount: f64,
    ) {
        self.assert_deposit_event(user, expected_lp_amount, expected_deposits);
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
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        (yusd_deposit, yaro_deposit): (f64, f64),
        (expected_yusd_reward, expected_yaro_reward): (f64, f64),
        expected_lp_amount: f64,
    ) {
        self.pool.assert_total_lp_less_or_equal_d();

        let (user_yusd_before, user_yaro_before, user_lp_amount_before) =
            snapshot_before.get_user_balances(user);
        let (user_yusd_after, user_yaro_after, user_lp_amount_after) =
            snapshot_after.get_user_balances(user);

        let expected_yusd_reward = float_to_uint(expected_yusd_reward, 7);
        let expected_yaro_reward = float_to_uint(expected_yaro_reward, 7);
        let yusd_deposit = float_to_uint(yusd_deposit, 7);
        let yaro_deposit = float_to_uint(yaro_deposit, 7);
        let expected_lp_amount = float_to_uint_sp(expected_lp_amount);

        let user_lp_diff = user_lp_amount_after - user_lp_amount_before;
        let user_yusd_diff = yusd_deposit - (user_yusd_before - user_yusd_after);
        let user_yaro_diff = yaro_deposit - (user_yaro_before - user_yaro_after);

        let pool_yusd_diff =
            yusd_deposit - (snapshot_after.pool_yusd_balance - snapshot_before.pool_yusd_balance);
        let pool_yaro_diff =
            yaro_deposit - (snapshot_after.pool_yaro_balance - snapshot_before.pool_yaro_balance);

        assert!(snapshot_before.total_lp_amount < snapshot_after.total_lp_amount);
        assert_eq!(
            snapshot_after.total_lp_amount - snapshot_before.total_lp_amount,
            user_lp_diff
        );
        assert!(snapshot_before.d < snapshot_after.d);
        assert_eq!(user_lp_diff, expected_lp_amount);

        assert_eq!(user_yusd_diff, expected_yusd_reward);
        assert_eq!(pool_yusd_diff, expected_yusd_reward);

        assert_eq!(user_yaro_diff, expected_yaro_reward);
        assert_eq!(pool_yaro_diff, expected_yaro_reward);
    }

    pub fn assert_withdraw(
        &self,
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        (expected_yusd_amount, expected_yaro_amount): (f64, f64),
        (expected_yusd_fee, expected_yaro_fee): (f64, f64),
        (expected_yusd_reward, expected_yaro_reward): (f64, f64),
        expected_user_withdraw_lp_diff: f64,
        (expected_yusd_admin_fee, expected_yaro_admin_fee): (f64, f64),
    ) {
        self.pool.assert_total_lp_less_or_equal_d();
        self.assert_withdraw_event(
            user,
            expected_user_withdraw_lp_diff,
            (expected_yusd_amount, expected_yaro_amount),
            (expected_yusd_fee, expected_yaro_fee),
        );

        let (user_yusd_before, user_yaro_before, user_lp_amount_before) =
            snapshot_before.get_user_balances(user);
        let (user_yusd_after, user_yaro_after, user_lp_amount_after) =
            snapshot_after.get_user_balances(user);

        let user_yaro_diff = user_yaro_after - user_yaro_before;
        let user_yusd_diff = user_yusd_after - user_yusd_before;
        let user_lp_diff = user_lp_amount_before - user_lp_amount_after;

        let expected_yusd_diff = float_to_uint(expected_yusd_amount + expected_yusd_reward, 7);
        let expected_yaro_diff = float_to_uint(expected_yaro_amount + expected_yaro_reward, 7);

        let expected_yusd_admin_fee = float_to_uint(expected_yusd_admin_fee, 7);
        let expected_yaro_admin_fee = float_to_uint(expected_yaro_admin_fee, 7);

        let pool_yaro_diff = snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance;
        let pool_yusd_diff = snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance;
        let expected_user_withdraw_lp_amount = float_to_uint_sp(expected_user_withdraw_lp_diff);

        let admin_yusd_fee_diff =
            snapshot_after.admin_yusd_fee_rewards - snapshot_before.admin_yusd_fee_rewards;
        let admin_yaro_fee_diff =
            snapshot_after.admin_yaro_fee_rewards - snapshot_before.admin_yaro_fee_rewards;

        assert_eq!(admin_yusd_fee_diff, expected_yusd_admin_fee);
        assert_eq!(admin_yaro_fee_diff, expected_yaro_admin_fee);

        assert!(snapshot_before.total_lp_amount > snapshot_after.total_lp_amount);
        let pool_lp_amount_diff = snapshot_before.total_lp_amount - snapshot_after.total_lp_amount;

        assert!(snapshot_before.d > snapshot_after.d);
        assert_eq!(user_lp_diff, pool_lp_amount_diff);
        assert_eq!(user_lp_diff, expected_user_withdraw_lp_amount);
        assert_eq!(pool_lp_amount_diff, expected_user_withdraw_lp_amount);

        if expected_yusd_fee != 0.0 && expected_yaro_fee != 0.0 {
            assert!(
                snapshot_before.acc_reward_yusd_per_share_p
                    < snapshot_after.acc_reward_yusd_per_share_p
            );
            assert!(
                snapshot_before.acc_reward_yaro_per_share_p
                    < snapshot_after.acc_reward_yaro_per_share_p
            );
        }

        assert_eq!(user_yusd_diff, expected_yusd_diff);
        assert_eq!(user_yaro_diff, expected_yaro_diff);
        assert_eq!(pool_yusd_diff, expected_yusd_diff);
        assert_eq!(pool_yaro_diff, expected_yaro_diff);
    }

    pub fn assert_claim(
        &self,
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        (yusd_reward, yaro_reward): (f64, f64),
    ) {
        self.pool.assert_total_lp_less_or_equal_d();
        if yusd_reward + yaro_reward != 0.0 {
            self.assert_claimed_reward_event(user, (yusd_reward, yaro_reward));
        }

        let (user_yusd_before, user_yaro_before, _) = snapshot_before.get_user_balances(user);
        let (user_yusd_after, user_yaro_after, _) = snapshot_after.get_user_balances(user);

        let user_yusd_diff = user_yusd_after - user_yusd_before;
        let user_yaro_diff = user_yaro_after - user_yaro_before;

        let pool_yusd_diff = snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance;
        let pool_yaro_diff = snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance;

        let yusd_reward = float_to_uint(yusd_reward, 7);
        let yaro_reward = float_to_uint(yaro_reward, 7);

        assert_eq!(user_yusd_diff, yusd_reward);
        assert_eq!(pool_yusd_diff, yusd_reward);
        assert_eq!(user_yaro_diff, yaro_reward);
        assert_eq!(pool_yaro_diff, yaro_reward);
    }

    pub fn assert_claim_admin_fee(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        (yusd_reward, yaro_reward): (f64, f64),
    ) {
        let yusd_reward = float_to_uint(yusd_reward, 7);
        let yaro_reward = float_to_uint(yaro_reward, 7);

        let admin_yaro_diff =
            snapshot_after.admin_yaro_balance - snapshot_before.admin_yaro_balance;
        let admin_yusd_diff =
            snapshot_after.admin_yusd_balance - snapshot_before.admin_yusd_balance;

        let pool_yaro_diff = snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance;
        let pool_yusd_diff = snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance;

        assert_rel_eq(admin_yaro_diff, yaro_reward, 1);
        assert_rel_eq(admin_yusd_diff, yusd_reward, 1);
        assert_rel_eq(pool_yaro_diff, yaro_reward, 1);
        assert_rel_eq(pool_yusd_diff, yusd_reward, 1);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assert_swap(
        &self,
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        sender: &User,
        recipient: &User,
        direction: Direction,
        amount: f64,
        expected_receive_amount: f64,
        expected_fee: f64,
    ) {
        self.pool.assert_total_lp_less_or_equal_d();

        self.assert_swapped_event(
            sender,
            recipient,
            direction.clone(),
            amount,
            expected_receive_amount,
            expected_fee,
        );

        let sender_tag = sender.tag;
        let recipient_tag = recipient.tag;

        let (from_token_tag, to_token_tag) = match direction {
            Direction::A2B => ("yusd", "yaro"),
            Direction::B2A => ("yaro", "yusd"),
        };

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
        deposit: (f64, f64),
        expected_rewards: (f64, f64),
        expected_lp_amount: f64,
    ) -> (Snapshot, Snapshot) {
        let snapshot_before = Snapshot::take(self);
        self.pool.deposit(user, deposit, 0.0);
        let snapshot_after = Snapshot::take(self);

        let title = format!(
            "Deposit {} yusd, {} yaro, expected lp: {expected_lp_amount}",
            deposit.0, deposit.1
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

        if expected_rewards != DOUBLE_ZERO {
            self.assert_claimed_reward_event(user, expected_rewards);
        }

        (snapshot_before, snapshot_after)
    }

    pub fn do_swap(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        direction: Direction,
        expected_receive_amount: f64,
        expected_fee: f64,
    ) -> (Snapshot, Snapshot) {
        let snapshot_before = Snapshot::take(self);
        self.pool.swap(
            sender,
            recipient,
            amount,
            receive_amount_min,
            direction.clone(),
        );
        let snapshot_after = Snapshot::take(self);

        let title = format!("Swap {amount} yusd => {expected_receive_amount} yaro");
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_swap(
            snapshot_before.clone(),
            snapshot_after.clone(),
            sender,
            recipient,
            direction,
            amount,
            expected_receive_amount,
            expected_fee,
        );

        (snapshot_before, snapshot_after)
    }

    pub fn do_claim(&self, user: &User, expected_rewards: (f64, f64)) {
        let snapshot_before = Snapshot::take(self);
        self.pool.claim_rewards(user);
        let snapshot_after = Snapshot::take(self);

        let title = format!("Claim rewards, expected {:?}", expected_rewards);
        snapshot_before.print_change_with(&snapshot_after, &title);

        self.assert_claim(snapshot_before, snapshot_after, user, expected_rewards);
    }

    pub fn do_claim_admin_fee(&self, expected_rewards: (f64, f64)) {
        let snapshot_before = Snapshot::take(self);
        self.pool.claim_admin_fee();
        let snapshot_after = Snapshot::take(self);

        let title = format!("Claim admin fee, expected {:?}", expected_rewards);
        snapshot_before.print_change_with(&snapshot_after, &title);

        TestingEnv::assert_claim_admin_fee(snapshot_before, snapshot_after, expected_rewards);
    }

    pub fn do_withdraw(
        &self,
        user: &User,
        withdraw_amount: f64,
        expected_withdraw_amounts: (f64, f64),
        expected_fee: (f64, f64),
        expected_rewards: (f64, f64),
        expected_user_lp_diff: f64,
        expected_admin_fee: (f64, f64),
    ) -> (Snapshot, Snapshot) {
        let snapshot_before = Snapshot::take(self);
        self.pool.withdraw(user, withdraw_amount);
        let snapshot_after = Snapshot::take(self);
        snapshot_before.print_change_with(&snapshot_after, "Withdraw");

        if expected_rewards != DOUBLE_ZERO {
            self.assert_claimed_reward_event(user, expected_rewards);
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
