use soroban_sdk::{Address, Env};

use crate::{
    contracts::three_pool::ThreeToken,
    contracts_wrappers::{
        EventAsserts, PoolFactory, PoolInfo, TestingEnv, TestingEnvConfig, ThreePool, Token, User,
    },
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

impl TestingEnv<3> for ThreePoolTestingEnv {
    type Snapshot = ThreePoolSnapshot;

    const TOKENS: [&'static str; 3] = ["a", "b", "c"];

    fn event_asserts(&self) -> &EventAsserts<3> {
        &self.event_asserts
    }

    fn users(&self) -> (&User, &User, &User) {
        (&self.alice, &self.bob, &self.admin)
    }

    fn tokens(&self) -> [&Token<impl Into<usize>>; 3] {
        [&self.token_a, &self.token_b, &self.token_c]
    }

    fn assert_total_lp_less_or_equal_d(&self) {
        self.pool.assert_total_lp_less_or_equal_d();
    }

    fn withdraw(&self, user: &User, withdraw_amount: f64) {
        self.pool.withdraw(user, withdraw_amount);
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

    fn deposit(&self, user: &User, deposit_amounts: [f64; 3], min_lp_amount: f64) {
        self.pool.deposit(user, deposit_amounts, min_lp_amount);
    }

    fn claim_admin_fee(&self) {
        self.pool.claim_admin_fee();
    }

    fn claim_rewards(&self, user: &User) {
        self.pool.claim_rewards(user);
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
}
