#[derive(Debug, Clone)]
pub struct TestingEnvConfig {
    /// default: `0.0`, from 0.0 to 100.0
    pub pool_fee_share_percentage: f64,
    /// default: `0.0`, from 0.0 to 100.0
    pub pool_admin_fee_percentage: f64,
    /// default: `100_000.0`
    pub admin_init_deposit: f64,
}

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
