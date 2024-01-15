use soroban_sdk::{testutils::Address as _, Address, Env};

use super::{Pool, Token, User};

#[derive(Debug, Clone)]
pub struct TestingEnvConfig {
    /// default: `0`
    pub pool_fee_share_bp: f64,

    pub pool_admin_fee: u128,

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
}

impl Default for TestingEnvConfig {
    fn default() -> Self {
        TestingEnvConfig {
            pool_fee_share_bp: 0.0,
            pool_admin_fee: 0,
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
        let alice = User::generate(env);
        let bob = User::generate(env);

        native_token.airdrop_user(&alice);
        native_token.airdrop_user(&bob);

        let (yusd_token, yaro_token, pool) = TestingEnvironment::create_tokens_and_pool(
            &env,
            &admin,
            config.pool_fee_share_bp,
            config.pool_admin_fee,
            (config.yusd_admin_deposit, config.yaro_admin_deposit),
        );

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
        }
    }

    #[inline]
    pub fn native_airdrop(&self, to: &Address) {
        self.native_token.airdrop(&to);
    }

    pub fn create_tokens_and_pool(
        env: &Env,
        admin: &Address,
        fee_share_bp: f64,
        admin_fee: u128,
        admin_deposits: (f64, f64),
    ) -> (Token, Token, Pool) {
        let token_a = Token::create(env, &admin);
        let token_b = Token::create(env, &admin);
        let fee_share_bp = ((fee_share_bp as f64) * 10_000.0) as u128;
        let pool = Pool::create(env, 20, &token_a.id, &token_b.id, fee_share_bp, admin_fee);

        token_a.airdrop_amount(&admin, admin_deposits.0 * 2.0);
        token_b.airdrop_amount(&admin, admin_deposits.1 * 2.0);

        if admin_deposits.0 > 0.0 || admin_deposits.1 > 0.0 {
            pool.deposit_by_id(&admin, admin_deposits, 0.0).unwrap();
        }

        (token_a, token_b, pool)
    }
}
