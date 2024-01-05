use soroban_env_common::StorageType;
use soroban_sdk::{Env, IntoVal, Symbol, TryFromVal, Val};

use crate::Error;

// ----------------------------------------- //

pub trait SymbolKey {
    const STORAGE_KEY: Symbol;
}

pub trait DataStorageType {
    const STORAGE_TYPE: StorageType;
}

pub trait ExtendTtlInfo {
    /// @see https://github.com/stellar/soroban-examples/blob/main/token/src/storage_types.rs#L8
    /// @see https://github.com/stellar/soroban-examples/blob/7a7cc6268ada55113ce0b82a3ae4405f7ec8b8f0/token/src/balance.rs#L2
    const EXTEND_TTL_AMOUNT: u32;
    const LIFETIME_THRESHOLD: u32;
}

// ----------------------------------------- //

pub trait SimpleSorobanData:
    TryFromVal<Env, Val> + IntoVal<Env, Val> + SymbolKey + SorobanData + Sized
{
    #[inline(always)]
    fn get(env: &Env) -> Result<Self, Error> {
        Self::get_by_key(env, &Self::STORAGE_KEY)
    }

    #[inline(always)]
    fn has(env: &Env) -> bool {
        Self::has_by_key(env, Self::STORAGE_KEY)
    }

    #[inline(always)]
    fn save(&self, env: &Env) {
        self.save_by_key(env, &Self::STORAGE_KEY);
    }

    #[inline(always)]
    fn update<F>(env: &Env, handler: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Error>,
    {
        Self::update_by_key(env, &Self::STORAGE_KEY, handler)
    }

    #[inline(always)]
    fn extend_ttl(env: &Env) {
        Self::extend_ttl_by_key(env, &Self::STORAGE_KEY);
    }
}

pub trait SorobanData:
    TryFromVal<Env, Val> + IntoVal<Env, Val> + DataStorageType + ExtendTtlInfo + Sized
{
    fn get_by_key<K: IntoVal<Env, Val>>(env: &Env, key: &K) -> Result<Self, Error> {
        let result = (match Self::STORAGE_TYPE {
            StorageType::Instance => env.storage().instance().get(key),
            StorageType::Temporary => env.storage().temporary().get(key),
            StorageType::Persistent => env.storage().persistent().get(key),
        })
        .ok_or(Error::Uninitialized)?;

        Self::extend_ttl_by_key(env, key);

        Ok(result)
    }

    fn save_by_key<K: IntoVal<Env, Val>>(&self, env: &Env, key: &K) {
        match Self::STORAGE_TYPE {
            StorageType::Instance => env.storage().instance().set(key, self),
            StorageType::Temporary => env.storage().temporary().set(key, self),
            StorageType::Persistent => env.storage().persistent().set(key, self),
        };

        Self::extend_ttl_by_key(env, key);
    }

    fn extend_ttl_by_key<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
        match Self::STORAGE_TYPE {
            StorageType::Instance => env
                .storage()
                .instance()
                .extend_ttl(Self::LIFETIME_THRESHOLD, Self::EXTEND_TTL_AMOUNT),
            StorageType::Temporary => {
                env.storage()
                    .temporary()
                    .extend_ttl(key, Self::LIFETIME_THRESHOLD, Self::EXTEND_TTL_AMOUNT)
            }
            StorageType::Persistent => {
                env.storage()
                    .persistent()
                    .extend_ttl(key, Self::LIFETIME_THRESHOLD, Self::EXTEND_TTL_AMOUNT)
            }
        }
    }

    #[inline]
    fn has_by_key<K: IntoVal<Env, Val>>(env: &Env, key: K) -> bool {
        Self::get_by_key(env, &key).is_ok()
    }

    fn update_by_key<F, K>(env: &Env, key: &K, handler: F) -> Result<(), Error>
    where
        K: IntoVal<Env, Val>,
        F: FnOnce(&mut Self) -> Result<(), Error>,
    {
        {
            let mut object = Self::get_by_key(env, key)?;

            handler(&mut object)?;

            object.save_by_key(env, key);

            Ok(())
        }
    }
}
