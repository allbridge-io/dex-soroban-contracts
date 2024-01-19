use soroban_sdk::{testutils::Address as _, Address, Env};

use super::{CallResult, Pool, PoolFactory, Token, User};

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

    pub fn with_yaro_admin_deposit(mut self, yaro_admin_deposit: f64) -> Self {
        self.yaro_admin_deposit = yaro_admin_deposit;
        self
    }

    pub fn with_yusd_admin_deposit(mut self, yusd_admin_deposit: f64) -> Self {
        self.yusd_admin_deposit = yusd_admin_deposit;
        self
    }

    pub fn with_pool_admin_fee(mut self, pool_admin_fee: u128) -> Self {
        self.pool_admin_fee = pool_admin_fee;
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
        let alice = User::generate(env);
        let bob = User::generate(env);

        let factory = PoolFactory::create(&env);

        native_token.airdrop_user(&alice);
        native_token.airdrop_user(&bob);

        let (yusd_token, yaro_token) = TestingEnvironment::generate_token_pair(env, &admin);
        let pool = TestingEnvironment::create_pool(
            &env,
            &factory,
            &admin,
            &yusd_token,
            &yaro_token,
            config.pool_fee_share_bp,
            config.pool_admin_fee,
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

    pub fn create_pool(
        env: &Env,
        factory: &PoolFactory,
        admin: &Address,
        token_a: &Token,
        token_b: &Token,
        fee_share_bp: f64,
        admin_fee: u128,
        admin_deposits: (f64, f64),
    ) -> CallResult<Pool> {
        let fee_share_bp = ((fee_share_bp as f64) * 10_000.0) as u128;
        let pool =
            factory.create_pair(admin, 20, &token_a.id, &token_b.id, fee_share_bp, admin_fee)?;

        let pool = Pool::new(env, pool);
        token_a.airdrop_amount(admin, admin_deposits.0 * 2.0);
        token_b.airdrop_amount(admin, admin_deposits.1 * 2.0);

        if admin_deposits.0 > 0.0 || admin_deposits.1 > 0.0 {
            pool.deposit_by_id(admin, admin_deposits, 0.0).unwrap();
        }

        Ok(pool)
    }
}
