use soroban_sdk::{Address, Env};

use crate::{
    contracts::two_pool::TwoToken,
    contracts_wrappers::{EventAsserts, PoolInfo, TestingEnv, TestingEnvConfig, Token},
    utils::percentage_to_bp,
};

use crate::contracts_wrappers::{PoolFactory, TwoPool, User};

use super::TwoPoolSnapshot;

pub struct TwoPoolTestingEnv {
    pub env: Env,
    pub event_asserts: EventAsserts<2>,

    pub admin: User,
    pub native_token: Token<TwoToken>,

    pub alice: User,
    pub bob: User,

    pub token_a: Token<TwoToken>,
    pub token_b: Token<TwoToken>,

    pub pool: TwoPool,
    pub factory: PoolFactory,
}

impl Default for TwoPoolTestingEnv {
    fn default() -> Self {
        Self::create(TestingEnvConfig::default())
    }
}

impl TestingEnv<2> for TwoPoolTestingEnv {
    type Snapshot = TwoPoolSnapshot;

    const TOKENS: [&'static str; 2] = ["a", "b"];

    fn event_asserts(&self) -> &EventAsserts<2> {
        &self.event_asserts
    }

    fn assert_total_lp_less_or_equal_d(&self) {
        self.pool.assert_total_lp_less_or_equal_d();
    }

    fn withdraw(&self, user: &User, withdraw_amount: f64) {
        self.pool.withdraw(user, withdraw_amount);
    }

    fn deposit(&self, user: &User, deposit_amounts: [f64; 2], min_lp_amount: f64) {
        self.pool.deposit(user, deposit_amounts, min_lp_amount);
    }

    fn swap<T: Into<usize> + Copy>(
        &self,
        sender: &User,
        recipient: &User,
        amount: f64,
        receive_amount_min: f64,
        token_from: &Token<T>,
        token_to: &Token<T>,
    ) {
        self.pool.swap(
            sender,
            recipient,
            amount,
            receive_amount_min,
            token_from,
            token_to,
        );
    }

    fn claim_admin_fee(&self) {
        self.pool.claim_admin_fee();
    }

    fn claim_rewards(&self, user: &User) {
        self.pool.claim_rewards(user);
    }

    fn users(&self) -> (&User, &User, &User) {
        (&self.alice, &self.bob, &self.admin)
    }

    fn tokens(&self) -> [&Token<impl Into<usize>>; 2] {
        [&self.token_a, &self.token_b]
    }

    fn pool_info(&self) -> PoolInfo {
        let info = self.pool.client.get_pool();

        PoolInfo {
            id: self.pool.id.clone(),
            d: self.pool.d(),
            a: info.a,
            acc_rewards_per_share_p: info.acc_rewards_per_share_p.0,
            admin_fee_amount: info.admin_fee_amount.0,
            admin_fee_share_bp: info.admin_fee_share_bp,
            fee_share_bp: info.fee_share_bp,
            token_balances: info.token_balances.0,
            tokens: info.tokens.0,
            tokens_decimals: info.tokens_decimals.0,
            total_lp_amount: info.total_lp_amount,
        }
    }

    fn get_user_deposit(&self, user_address: &Address) -> crate::contracts_wrappers::UserDeposit {
        let deposit = self.pool.client.get_user_deposit(user_address);

        crate::contracts_wrappers::UserDeposit {
            reward_debts: deposit.reward_debts.0,
            lp_amount: deposit.lp_amount,
        }
    }
}

impl TwoPoolTestingEnv {
    pub fn create(config: TestingEnvConfig) -> TwoPoolTestingEnv {
        let env = Env::default();

        env.mock_all_auths();
        env.budget().reset_limits(u64::MAX, u64::MAX);

        let admin = User::generate(&env, "admin");
        let native_token = Token::create(&env, admin.as_ref(), TwoToken::A, "native");
        let alice = User::generate(&env, "alice");
        let bob = User::generate(&env, "bob");

        let factory = PoolFactory::create(&env, admin.as_ref());

        native_token.default_airdrop(&alice);
        native_token.default_airdrop(&bob);

        let (a_token, b_token) = TwoPoolTestingEnv::generate_token_pair(&env, admin.as_ref());
        let pool = TwoPoolTestingEnv::create_pool(
            &env,
            &factory,
            &admin,
            &a_token,
            &b_token,
            config.pool_fee_share_percentage,
            config.pool_admin_fee_percentage,
            config.admin_init_deposit,
        );

        a_token.default_airdrop(&admin);
        b_token.default_airdrop(&admin);

        a_token.default_airdrop(&alice);
        b_token.default_airdrop(&alice);

        a_token.default_airdrop(&bob);
        b_token.default_airdrop(&bob);

        TwoPoolTestingEnv {
            event_asserts: EventAsserts(env.clone()),
            env,

            admin,
            native_token,

            alice,
            bob,

            token_b: b_token,
            token_a: a_token,
            pool,
            factory,
        }
    }

    pub fn get_token(&self, pool_token: TwoToken) -> &Token<TwoToken> {
        match pool_token {
            TwoToken::A => &self.token_a,
            TwoToken::B => &self.token_b,
        }
    }

    pub fn clear_mock_auth(&self) -> &Self {
        self.env.mock_auths(&[]);
        self
    }

    pub fn generate_token_pair(env: &Env, admin: &Address) -> (Token<TwoToken>, Token<TwoToken>) {
        let token_a = Token::create(env, admin, TwoToken::A, "a");
        let token_b = Token::create(env, admin, TwoToken::B, "b");

        (token_a, token_b)
    }

    fn create_pool(
        env: &Env,
        factory: &PoolFactory,
        admin: &User,
        token_a: &Token<TwoToken>,
        token_b: &Token<TwoToken>,
        fee_share_percentage: f64,
        admin_fee_percentage: f64,
        admin_init_deposit: f64,
    ) -> TwoPool {
        let fee_share_bp = percentage_to_bp(fee_share_percentage);
        let admin_fee_bp = percentage_to_bp(admin_fee_percentage);
        let a = 20;
        let pool = factory.create_pool(
            admin.as_ref(),
            a,
            [token_a.id.clone(), token_b.id.clone()],
            fee_share_bp,
            admin_fee_bp,
        );

        let pool = TwoPool::new(env, pool);

        pool.assert_initialization(a, fee_share_bp, admin_fee_bp);

        token_a.airdrop(admin, admin_init_deposit * 2.0);
        token_b.airdrop(admin, admin_init_deposit * 2.0);

        if admin_init_deposit > 0.0 {
            pool.deposit(admin, [admin_init_deposit, admin_init_deposit], 0.0);
        }

        pool
    }
}
