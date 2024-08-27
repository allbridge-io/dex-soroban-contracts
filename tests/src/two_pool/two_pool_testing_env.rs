use soroban_sdk::{Address, Env};

use crate::{
    contracts::two_pool::TwoToken,
    utils::{EventAsserts, PoolClient, TestingEnv, TestingEnvConfig, Token},
};

use crate::utils::{PoolFactory, TwoPool, User};

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

    fn pool_client(&self) -> &impl PoolClient<2> {
        &self.pool
    }

    fn event_asserts(&self) -> &EventAsserts<2> {
        &self.event_asserts
    }

    fn users(&self) -> (&User, &User, &User) {
        (&self.alice, &self.bob, &self.admin)
    }

    fn tokens(&self) -> [&Token<impl Into<usize>>; 2] {
        [&self.token_a, &self.token_b]
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

        let tokens = TwoPoolTestingEnv::generate_token_pair(&env, admin.as_ref());
        let pool = TwoPoolTestingEnv::create_pool(
            &env,
            &factory,
            &admin,
            &tokens,
            config.pool_fee_share_percentage,
            config.pool_admin_fee_percentage,
            config.admin_init_deposit,
        );

        for token in tokens.iter() {
            token.default_airdrop(&admin);
            token.default_airdrop(&alice);
            token.default_airdrop(&bob);
        }

        let [token_a, token_b] = tokens;

        TwoPoolTestingEnv {
            event_asserts: EventAsserts(env.clone()),
            env,

            admin,
            native_token,

            alice,
            bob,

            token_a,
            token_b,
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

    pub fn generate_token_pair(env: &Env, admin: &Address) -> [Token<TwoToken>; 2] {
        let token_a = Token::create(env, admin, TwoToken::A, "a");
        let token_b = Token::create(env, admin, TwoToken::B, "b");

        [token_a, token_b]
    }
}
