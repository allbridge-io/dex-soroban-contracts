use soroban_sdk::Env;

use crate::consts::{INSTANCE_EXTEND_TTL_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};

#[inline(always)]
pub fn extend_ttl_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_EXTEND_TTL_AMOUNT);
}
