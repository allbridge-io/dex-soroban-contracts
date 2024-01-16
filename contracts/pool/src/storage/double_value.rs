use soroban_sdk::{
    testutils::arbitrary::SorobanArbitrary, xdr::ScVal, ConversionError, Env, TryFromVal,
    TryIntoVal, Val,
};

use core::{
    fmt::Debug,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use super::pool::Token;

#[derive(Clone)]
pub struct DoubleValue {
    env: Env,
    data: [u128; 2],
}

impl Debug for DoubleValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DoubleValue({:?})", self.data)?;
        Ok(())
    }
}

impl Index<usize> for DoubleValue {
    type Output = u128;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for DoubleValue {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Index<Token> for DoubleValue {
    type Output = u128;

    fn index(&self, index: Token) -> &Self::Output {
        &self.data[index as usize]
    }
}

impl IndexMut<Token> for DoubleValue {
    fn index_mut(&mut self, index: Token) -> &mut Self::Output {
        &mut self.data[index as usize]
    }
}

impl DoubleValue {
    pub fn default(env: &Env) -> Self {
        Self {
            env: env.clone().into(),
            data: Default::default(),
        }
    }

    pub fn new(env: &Env, data: [u128; 2]) -> Self {
        Self {
            env: env.clone().into(),
            data,
        }
    }
}

impl Deref for DoubleValue {
    type Target = [u128; 2];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for DoubleValue {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl SorobanArbitrary for DoubleValue {
    type Prototype = (u128, u128);
}

impl TryFromVal<Env, Val> for DoubleValue {
    type Error = ConversionError;

    fn try_from_val(env: &Env, v: &Val) -> Result<Self, Self::Error> {
        let tuple: (u128, u128) = TryFromVal::try_from_val(env, v)?;

        Ok(Self {
            env: env.clone().into(),
            data: [tuple.0, tuple.1],
        })
    }
}

impl TryFromVal<Env, DoubleValue> for Val {
    type Error = ConversionError;

    fn try_from_val(env: &Env, v: &DoubleValue) -> Result<Self, Self::Error> {
        (v.data[0], v.data[1])
            .try_into_val(env)
            .map_err(|_| ConversionError)
    }
}

impl TryFromVal<Env, (u128, u128)> for DoubleValue {
    type Error = ConversionError;

    fn try_from_val(env: &Env, v: &(u128, u128)) -> Result<Self, Self::Error> {
        Ok(Self {
            env: env.clone().into(),
            data: [v.0, v.1],
        })
    }
}

#[cfg(not(target_family = "wasm"))]
impl TryFrom<&DoubleValue> for ScVal {
    type Error = ConversionError;

    fn try_from(v: &DoubleValue) -> Result<Self, ConversionError> {
        let val = Val::try_from_val(&v.env, &(v.data[0], v.data[1]))?;

        ScVal::try_from_val(&v.env, &val)
    }
}
