use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    contracts::pool::{Deposit, Direction, RewardsClaimed, Swapped, Withdraw},
    utils::{assert_rel_eq, float_to_uint, float_to_uint_sp},
};

use super::{get_latest_event, CallResult, Pool, PoolFactory, Snapshot, Token, User};

#[derive(Debug, Clone)]
pub struct TestingEnvConfig {
    /// default: `0`
    pub pool_fee_share: f64,
    /// default: `0`
    pub pool_admin_fee: f64,
    /// default: `100_000.0`
    pub admin_init_deposit: f64,
}

impl TestingEnvConfig {
    pub fn with_admin_init_deposit(mut self, admin_init_deposit: f64) -> Self {
        self.admin_init_deposit = admin_init_deposit;
        self
    }

    pub fn with_pool_admin_fee(mut self, pool_admin_fee: f64) -> Self {
        self.pool_admin_fee = pool_admin_fee;
        self
    }

    pub fn with_pool_fee_share(mut self, fee_share: f64) -> Self {
        self.pool_fee_share = fee_share;
        self
    }
}

impl Default for TestingEnvConfig {
    fn default() -> Self {
        TestingEnvConfig {
            pool_fee_share: 0.0,
            pool_admin_fee: 0.0,
            admin_init_deposit: 100_000.0,
        }
    }
}

#[allow(dead_code)]
pub struct TestingEnvironment {
    config: TestingEnvConfig,

    pub admin: Address,

    pub native_token: Token,

    pub alice: User,
    pub bob: User,

    pub yaro_token: Token,
    pub yusd_token: Token,

    pub pool: Pool,
    pub factory: PoolFactory,
}

impl TestingEnvironment {
    pub fn default(env: &Env) -> TestingEnvironment {
        Self::create(env, TestingEnvConfig::default())
    }

    pub fn create(env: &Env, config: TestingEnvConfig) -> TestingEnvironment {
        env.mock_all_auths();
        env.budget().reset_limits(u64::MAX, u64::MAX);

        let admin = Address::generate(env);
        let native_token = Token::create(env, &admin);
        let alice = User::generate(env, "alice");
        let bob = User::generate(env, "bob");

        let factory = PoolFactory::create(env, &admin);

        native_token.airdrop_user(&alice);
        native_token.airdrop_user(&bob);

        let (yusd_token, yaro_token) = TestingEnvironment::generate_token_pair(env, &admin);
        let pool = TestingEnvironment::create_pool(
            env,
            &factory,
            &admin,
            &yusd_token,
            &yaro_token,
            config.pool_fee_share,
            config.pool_admin_fee,
            config.admin_init_deposit,
        )
        .unwrap();

        yusd_token.airdrop_user(&alice);
        yusd_token.airdrop_user(&bob);

        yaro_token.airdrop_user(&alice);
        yaro_token.airdrop_user(&bob);

        TestingEnvironment {
            config,

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

    pub fn generate_token_pair(env: &Env, admin: &Address) -> (Token, Token) {
        let token_a = Token::create(env, admin);
        let token_b = Token::create(env, admin);

        (token_a, token_b)
    }

    pub fn get_tokens_by_direction(&self, direction: Direction) -> (&Token, &Token) {
        match direction {
            Direction::A2B => (&self.yusd_token, &self.yaro_token),
            Direction::B2A => (&self.yaro_token, &self.yusd_token),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_pool(
        env: &Env,
        factory: &PoolFactory,
        admin: &Address,
        token_a: &Token,
        token_b: &Token,
        fee_share: f64,
        admin_fee: f64,
        admin_init_deposit: f64,
    ) -> CallResult<Pool> {
        let fee_share_bp = Pool::convert_to_bp(fee_share);
        let admin_fee_bp = Pool::convert_to_bp(admin_fee);
        let a = 20;
        let pool = factory.create_pair(
            admin,
            a,
            &token_a.id,
            &token_b.id,
            fee_share_bp,
            admin_fee_bp,
        )?;

        let pool = Pool::new(env, pool);

        pool.assert_initialization(a, fee_share_bp, admin_fee_bp);

        token_a.airdrop_amount(admin, admin_init_deposit * 2.0);
        token_b.airdrop_amount(admin, admin_init_deposit * 2.0);

        if admin_init_deposit > 0.0 {
            pool.deposit_with_address(admin, (admin_init_deposit, admin_init_deposit), 0.0)
                .unwrap();
        }

        Ok(pool)
    }

    pub fn assert_claimed_reward_event(
        env: &Env,
        expected_user: &User,
        (expected_yusd_reward, expected_yaro_reward): (f64, f64),
    ) {
        let rewards_claimed = get_latest_event::<RewardsClaimed>(env).unwrap();

        assert_eq!(rewards_claimed.user, expected_user.as_address());
        assert_rel_eq(
            rewards_claimed.rewards.0,
            float_to_uint(expected_yusd_reward, 7),
            10,
        );
        assert_rel_eq(
            rewards_claimed.rewards.1,
            float_to_uint(expected_yaro_reward, 7),
            10,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assert_swapped_event(
        &self,
        env: &Env,
        sender: &User,
        recipient: &User,
        directin: Direction,
        from_amount: f64,
        expected_to_amount: u128,
        extected_fee: u128,
    ) {
        let swapped = get_latest_event::<Swapped>(env).unwrap();

        let (from_token, to_token) = match directin {
            Direction::A2B => (self.yusd_token.as_address(), self.yaro_token.as_address()),
            Direction::B2A => (self.yaro_token.as_address(), self.yusd_token.as_address()),
        };

        assert_eq!(swapped.sender, sender.as_address());
        assert_eq!(swapped.recipient, recipient.as_address());

        assert_eq!(swapped.from_amount, float_to_uint(from_amount, 7));
        assert_eq!(swapped.to_amount, expected_to_amount);
        assert_rel_eq(swapped.fee, extected_fee, 10);

        assert_eq!(swapped.from_token, from_token);
        assert_eq!(swapped.to_token, to_token);
    }

    pub fn assert_withdraw_event(
        env: &Env,
        expected_user: &User,
        lp_amount: f64,
        (yusd_amount, yaro_amount): (f64, f64),
    ) {
        let withdraw = get_latest_event::<Withdraw>(env).unwrap();

        assert_eq!(withdraw.user, expected_user.as_address());
        assert_rel_eq(withdraw.amounts.0, float_to_uint_sp(yusd_amount), 2);
        assert_rel_eq(withdraw.amounts.1, float_to_uint_sp(yaro_amount), 2);
        assert_eq!(withdraw.lp_amount, float_to_uint_sp(lp_amount));
    }

    pub fn assert_deposit_event(
        env: &Env,
        expected_user: &User,
        expected_lp_amount: f64,
        (yusd_deposit, yaro_deposit): (f64, f64),
    ) {
        let deposit = get_latest_event::<Deposit>(env).unwrap();

        assert_eq!(deposit.user, expected_user.as_address());
        assert_eq!(deposit.amounts.0, float_to_uint(yusd_deposit, 7));
        assert_eq!(deposit.amounts.1, float_to_uint(yaro_deposit, 7));
        assert_rel_eq(float_to_uint_sp(expected_lp_amount), deposit.lp_amount, 10);
    }

    pub fn assert_deposit(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        (yusd_deposit, yaro_deposit): (f64, f64),
        (expected_yusd_reward, expected_yaro_reward): (f64, f64),
        expected_lp_amount: f64,
    ) {
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
        assert_rel_eq(user_lp_diff, expected_lp_amount, 5);

        assert_eq!(user_yusd_diff, expected_yusd_reward);
        assert_eq!(pool_yusd_diff, expected_yusd_reward);

        assert_eq!(user_yaro_diff, expected_yaro_reward);
        assert_eq!(pool_yaro_diff, expected_yaro_reward);
    }

    pub fn assert_withdraw(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        (yusd_amount, yaro_amount): (f64, f64),
        (expected_yusd_reward, expected_yaro_reward): (f64, f64),
        expected_user_withdraw_lp_diff: f64,
    ) {
        let (user_yusd_before, user_yaro_before, user_lp_amount_before) =
            snapshot_before.get_user_balances(user);
        let (user_yusd_after, user_yaro_after, user_lp_amount_after) =
            snapshot_after.get_user_balances(user);

        let user_yaro_diff = user_yaro_after - user_yaro_before;
        let user_yusd_diff = user_yusd_after - user_yusd_before;
        let user_lp_diff = user_lp_amount_before - user_lp_amount_after;

        let expected_yusd_diff = float_to_uint(yusd_amount + expected_yusd_reward, 7);
        let expected_yaro_diff = float_to_uint(yaro_amount + expected_yaro_reward, 7);

        let pool_yaro_diff = snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance;
        let pool_yusd_diff = snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance;
        let expected_user_withdraw_lp_amount = float_to_uint_sp(expected_user_withdraw_lp_diff);

        assert!(snapshot_before.total_lp_amount > snapshot_after.total_lp_amount);
        let pool_lp_amount_diff = snapshot_before.total_lp_amount - snapshot_after.total_lp_amount;

        assert!(snapshot_before.d > snapshot_after.d);
        assert_eq!(user_lp_diff, pool_lp_amount_diff);
        assert_eq!(user_lp_diff, expected_user_withdraw_lp_amount);
        assert_eq!(pool_lp_amount_diff, expected_user_withdraw_lp_amount);

        // 10000 with 7 precision => 0.001
        assert_rel_eq(user_yusd_diff, expected_yusd_diff, 10000);
        assert_rel_eq(user_yaro_diff, expected_yaro_diff, 10000);
        assert_rel_eq(pool_yusd_diff, expected_yusd_diff, 10000);
        assert_rel_eq(pool_yaro_diff, expected_yaro_diff, 10000);
    }

    pub fn assert_claim(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        (yusd_reward, yaro_reward): (f64, f64),
    ) {
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

        assert_rel_eq(admin_yaro_diff, yaro_reward, 5);
        assert_rel_eq(admin_yusd_diff, yusd_reward, 5);
        assert_rel_eq(pool_yaro_diff, yaro_reward, 5);
        assert_rel_eq(pool_yusd_diff, yusd_reward, 5);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assert_swap(
        &self,
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        sender: &User,
        recipient: &User,
        directin: Direction,
        amount: f64,
        receive_amount_min: f64,
        expected_receive_amount: u128,
    ) {
        let sender_tag = sender.tag;
        let recipient_tag = recipient.tag;

        let (from_token_tag, to_token_tag) = match directin {
            Direction::A2B => ("yusd", "yaro"),
            Direction::B2A => ("yaro", "yusd"),
        };

        let sender_balance_key = format!("{sender_tag}_{from_token_tag}_balance");
        let recipient_balance_key = format!("{recipient_tag}_{to_token_tag}_balance");
        let pool_from_balance_key = format!("pool_{from_token_tag}_balance");
        let pool_to_balance_key = format!("pool_{to_token_tag}_balance");
        let acc_reward_token_to_per_share_p_key = format!("acc_reward_{to_token_tag}_per_share_p");

        let receive_amount_min = float_to_uint(receive_amount_min, 7);
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
                >= snapshot_before[&acc_reward_token_to_per_share_p_key]
        );
        assert!(recipient_to_token_diff >= receive_amount_min);
        assert!(recipient_to_token_diff <= expected_receive_amount);

        assert!(pool_to_token_diff >= receive_amount_min);
        assert!(pool_to_token_diff <= expected_receive_amount);

        assert_eq!(sender_from_token_diff, amount);
        assert_eq!(pool_from_token_diff, amount);
    }
}
