use soroban_sdk::{Address, Env};

use crate::{
    contracts::three_pool::ThreeToken,
    utils::{
        EventAsserts, PoolClient, PoolFactory, TestingEnv, TestingEnvConfig, ThreePool, Token, User,
    },
};

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

    fn pool_client(&self) -> &impl PoolClient<3> {
        &self.pool
    }

    fn event_asserts(&self) -> &EventAsserts<3> {
        &self.event_asserts
    }

    fn users(&self) -> (&User, &User, &User) {
        (&self.alice, &self.bob, &self.admin)
    }

    fn tokens(&self) -> [&Token<impl Into<usize>>; 3] {
        [&self.token_a, &self.token_b, &self.token_c]
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

        let tokens = ThreePoolTestingEnv::generate_tokens(&env, admin.as_ref());

        let pool = ThreePoolTestingEnv::create_pool(
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

        let [token_a, token_b, token_c] = tokens;

        ThreePoolTestingEnv {
            event_asserts: EventAsserts(env.clone()),
            env,

            admin,
            native_token,

            alice,
            bob,

            token_a,
            token_b,
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

    pub fn generate_tokens(env: &Env, admin: &Address) -> [Token<ThreeToken>; 3] {
        let token_a = Token::create(env, admin, ThreeToken::A, "a");
        let token_b = Token::create(env, admin, ThreeToken::B, "b");
        let token_c = Token::create(env, admin, ThreeToken::C, "c");

        [token_a, token_b, token_c]
    }
}
