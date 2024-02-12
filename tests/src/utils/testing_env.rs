use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    contracts::pool::{Deposit, Direction, RewardsClaimed, Swapped, Withdraw},
    utils::{assert_rel_eq, float_to_int, float_to_int_sp},
};

use super::{
    assert_rel_eq_f64, get_latest_event, int_to_float, CallResult, Pool, PoolFactory, Snapshot,
    Token, User,
};

#[derive(Debug, Clone)]
pub struct TestingEnvConfig {
    /// default: `0`
    pub pool_fee_share_bp: f64,

    pub pool_admin_fee_bp: f64,

    /// default: `100_000.0`
    pub yaro_admin_deposit: f64,
    /// default: `100_000.0`
    pub yusd_admin_deposit: f64,
}

impl TestingEnvConfig {
    pub fn with_pool_fee_share_bp(mut self, fee_share_bp: f64) -> Self {
        self.pool_fee_share_bp = fee_share_bp;
        self
    }

    pub fn with_yaro_admin_deposit(mut self, yaro_admin_deposit: f64) -> Self {
        self.yaro_admin_deposit = yaro_admin_deposit;
        self
    }

    pub fn with_yusd_admin_deposit(mut self, yusd_admin_deposit: f64) -> Self {
        self.yusd_admin_deposit = yusd_admin_deposit;
        self
    }

    pub fn with_pool_admin_fee(mut self, pool_admin_fee: f64) -> Self {
        self.pool_admin_fee_bp = pool_admin_fee;
        self
    }
}

impl Default for TestingEnvConfig {
    fn default() -> Self {
        TestingEnvConfig {
            pool_fee_share_bp: 0.0,
            pool_admin_fee_bp: 0.0,
            yaro_admin_deposit: 100_000.0,
            yusd_admin_deposit: 100_000.0,
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
            config.pool_fee_share_bp,
            config.pool_admin_fee_bp,
            (config.yusd_admin_deposit, config.yaro_admin_deposit),
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
        fee_share_bp: f64,
        admin_fee: f64,
        admin_deposits: (f64, f64),
    ) -> CallResult<Pool> {
        let fee_share_bp = (fee_share_bp * 10_000.0) as u128;
        let admin_fee = (admin_fee * 10_000.0) as u128;
        let a = 20;
        let pool =
            factory.create_pair(admin, a, &token_a.id, &token_b.id, fee_share_bp, admin_fee)?;

        let pool = Pool::new(env, pool);

        pool.assert_initialization(a, fee_share_bp, admin_fee);

        token_a.airdrop_amount(admin, admin_deposits.0 * 2.0);
        token_b.airdrop_amount(admin, admin_deposits.1 * 2.0);

        if admin_deposits.0 > 0.0 || admin_deposits.1 > 0.0 {
            pool.deposit_by_id(admin, admin_deposits, 0.0).unwrap();
        }

        Ok(pool)
    }

    pub fn assert_claimed_reward_event(env: &Env, expected_user: &User, rewards: (f64, f64)) {
        let rewards_claimed = get_latest_event::<RewardsClaimed>(env).unwrap();

        println!("reward {:?}", rewards_claimed.rewards);
        assert_eq!(rewards_claimed.user, expected_user.as_address());
        assert_rel_eq(rewards_claimed.rewards.0, float_to_int(rewards.0, 7), 10);
        assert_rel_eq(rewards_claimed.rewards.1, float_to_int(rewards.1, 7), 10);
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

        assert_eq!(swapped.from_amount, float_to_int(from_amount, 7));
        assert_eq!(swapped.to_amount, expected_to_amount);
        assert_rel_eq(swapped.fee, extected_fee, 10);

        assert_eq!(swapped.from_token, from_token);
        assert_eq!(swapped.to_token, to_token);
    }

    pub fn assert_withdraw_event(
        env: &Env,
        expected_user: &User,
        lp_amount: f64,
        expected_amount: (f64, f64),
    ) {
        let withdraw = get_latest_event::<Withdraw>(env).unwrap();
        println!("withdraw {:?}", withdraw.amounts);

        assert_eq!(withdraw.user, expected_user.as_address());
        assert_rel_eq(withdraw.amounts.0, float_to_int_sp(expected_amount.0), 2);
        assert_rel_eq(withdraw.amounts.1, float_to_int_sp(expected_amount.1), 2);
        assert_eq!(withdraw.lp_amount, float_to_int_sp(lp_amount));
    }

    pub fn assert_deposit_event(
        env: &Env,
        expected_user: &User,
        expected_lp_amount: f64,
        deposits: (f64, f64),
    ) {
        let deposit = get_latest_event::<Deposit>(env).unwrap();

        assert_eq!(deposit.user, expected_user.as_address());
        assert_eq!(int_to_float(deposit.amounts.0, 7), deposits.0);
        assert_eq!(int_to_float(deposit.amounts.1, 7), deposits.1);
        assert_rel_eq(float_to_int_sp(expected_lp_amount), deposit.lp_amount, 10);
    }

    pub fn assert_deposit(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        deposits: (f64, f64),
        expected_rewards: (f64, f64),
        expected_lp_amount: f64,
    ) {
        let (user_yusd_before, user_yaro_before, user_lp_amount_before) =
            snapshot_before.get_user_balances(user);
        let (user_yusd_after, user_yaro_after, user_lp_amount_after) =
            snapshot_after.get_user_balances(user);

        let total_deposits = deposits.0 + deposits.1;
        let (yusd_deposit, yaro_deposit) = deposits;
        let (expected_yusd_reward, expected_yaro_reward) = expected_rewards;

        let lp_diff = user_lp_amount_after - user_lp_amount_before;
        let user_yusd_diff = yusd_deposit - int_to_float(user_yusd_before - user_yusd_after, 7);
        let user_yaro_diff = yaro_deposit - int_to_float(user_yaro_before - user_yaro_after, 7);

        let pool_yusd_diff = yusd_deposit
            - int_to_float(
                snapshot_after.pool_yusd_balance - snapshot_before.pool_yusd_balance,
                7,
            );
        
        // TODO: Exactly equal here
        assert_rel_eq_f64(user_yusd_diff, expected_yusd_reward, 0.0001);
        assert_rel_eq_f64(pool_yusd_diff, expected_yusd_reward, 0.0001);

        let pool_yaro_diff = yaro_deposit
            - int_to_float(
                snapshot_after.pool_yaro_balance - snapshot_before.pool_yaro_balance,
                7,
            );

        assert!(expected_lp_amount <= total_deposits);

        // TODO: Also compare pool LP diff with user LP diff
        assert!(snapshot_before.total_lp_amount < snapshot_after.total_lp_amount);
        assert!(snapshot_before.d < snapshot_after.d);

        // TODO: Exactly equal here
        assert_rel_eq_f64(user_yaro_diff, expected_yaro_reward, 0.0001);
        assert_rel_eq_f64(pool_yaro_diff, expected_yaro_reward, 0.0001);

        assert_rel_eq(lp_diff, float_to_int_sp(expected_lp_amount), 5);
    }

    pub fn assert_withdraw(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        expected_amount: (f64, f64),
        expected_rewards: (f64, f64),
        expected_withdraw_lp_amount: f64,
    ) {
        let (user_yusd_before, user_yaro_before, user_lp_amount_before) =
            snapshot_before.get_user_balances(user);
        let (user_yusd_after, user_yaro_after, user_lp_amount_after) =
            snapshot_after.get_user_balances(user);

        let user_yaro_diff = user_yaro_after - user_yaro_before;
        let user_yusd_diff = user_yusd_after - user_yusd_before;
        let lp_diff = user_lp_amount_before - user_lp_amount_after;

        let pool_yaro_diff = int_to_float(
            snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance,
            7,
        );
        let pool_yusd_diff = int_to_float(
            snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance,
            7,
        );

        let total_lp_amount_diff = snapshot_before.total_lp_amount - snapshot_after.total_lp_amount
            + float_to_int_sp(expected_rewards.0 + expected_rewards.1);

        assert!(snapshot_before.total_lp_amount > snapshot_after.total_lp_amount);
        assert!(snapshot_before.d > snapshot_after.d);

        assert!(lp_diff <= float_to_int_sp(expected_withdraw_lp_amount));
        assert_rel_eq(
            total_lp_amount_diff,
            float_to_int_sp(expected_withdraw_lp_amount + expected_rewards.0 + expected_rewards.1),
            float_to_int_sp(0.1),
        );
        assert_rel_eq(
            user_yusd_diff,
            float_to_int(expected_amount.0 + expected_rewards.0, 7),
            10000,
        );
        assert_rel_eq(
            user_yaro_diff,
            float_to_int(expected_amount.1 + expected_rewards.1, 7),
            10000,
        );
        assert_rel_eq(
            float_to_int(pool_yusd_diff, 7),
            float_to_int(expected_amount.0 + expected_rewards.0, 7),
            10000,
        );
        assert_rel_eq(
            float_to_int(pool_yaro_diff, 7),
            float_to_int(expected_amount.1 + expected_rewards.1, 7),
            10000,
        );
    }

    pub fn assert_claim(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        user: &User,
        expected_rewards: (f64, f64),
    ) {
        let (user_yusd_before, user_yaro_before, _) = snapshot_before.get_user_balances(user);
        let (user_yusd_after, user_yaro_after, _) = snapshot_after.get_user_balances(user);

        let user_yusd_diff = user_yusd_after - user_yusd_before;
        let user_yaro_diff = user_yaro_after - user_yaro_before;

        let pool_yusd_diff = snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance;
        let pool_yaro_diff = snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance;

        assert_eq!(int_to_float(user_yusd_diff, 7), expected_rewards.0);
        assert_eq!(int_to_float(pool_yusd_diff, 7), expected_rewards.0);

        assert_eq!(int_to_float(user_yaro_diff, 7), expected_rewards.1);
        assert_eq!(int_to_float(pool_yaro_diff, 7), expected_rewards.1);
    }

    pub fn assert_claim_admin_fee(
        snapshot_before: Snapshot,
        snapshot_after: Snapshot,
        expected_admin_rewards: (f64, f64),
    ) {
        let admin_yaro_diff = int_to_float(
            snapshot_after.admin_yaro_balance - snapshot_before.admin_yaro_balance,
            7,
        );
        let admin_yusd_diff = int_to_float(
            snapshot_after.admin_yusd_balance - snapshot_before.admin_yusd_balance,
            7,
        );

        let pool_yaro_diff = int_to_float(
            snapshot_before.pool_yaro_balance - snapshot_after.pool_yaro_balance,
            7,
        );
        let pool_yusd_diff = int_to_float(
            snapshot_before.pool_yusd_balance - snapshot_after.pool_yusd_balance,
            7,
        );

        assert_rel_eq_f64(admin_yaro_diff, expected_admin_rewards.1, 0.0001);
        assert_rel_eq_f64(admin_yusd_diff, expected_admin_rewards.0, 0.0001);
        assert_rel_eq_f64(pool_yaro_diff, expected_admin_rewards.1, 0.0001);
        assert_rel_eq_f64(pool_yusd_diff, expected_admin_rewards.0, 0.0001);
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

        let expected_receive_amount_f64 = int_to_float(expected_receive_amount, 7);

        let sender_from_token_diff = int_to_float(
            snapshot_before[&sender_balance_key] - snapshot_after[&sender_balance_key],
            7,
        );

        let recipient_to_token_diff = int_to_float(
            snapshot_after[&recipient_balance_key] - snapshot_before[&recipient_balance_key],
            7,
        );

        let pool_from_token_diff = int_to_float(
            snapshot_after[&pool_from_balance_key] - snapshot_before[&pool_from_balance_key],
            7,
        );
        let pool_to_token_diff = int_to_float(
            snapshot_before[&pool_to_balance_key] - snapshot_after[&pool_to_balance_key],
            7,
        );

        assert!(
            snapshot_after[&acc_reward_token_to_per_share_p_key]
                >= snapshot_before[&acc_reward_token_to_per_share_p_key]
        );
        assert!(recipient_to_token_diff >= receive_amount_min);
        assert!(recipient_to_token_diff <= expected_receive_amount_f64);

        assert!(pool_to_token_diff >= receive_amount_min);
        assert!(pool_to_token_diff <= expected_receive_amount_f64);

        assert_eq!(sender_from_token_diff, amount);
        assert_eq!(pool_from_token_diff, amount);
    }
}
