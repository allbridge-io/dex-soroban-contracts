use soroban_sdk::{contracttype, Address, Env, Vec};

use core::ops::{Deref, Sub};

#[macro_export]
macro_rules! sized_array {
    ($name:ident, $inner_type:ident) => {
        #[contracttype]
        #[derive(Debug, Clone)]
        pub struct $name(Vec<$inner_type>);

        impl $name {
            #[inline]
            pub fn from_array<const N: usize>(env: &Env, items: [$inner_type; N]) -> Self {
                Self::from_vec(Vec::from_array(env, items))
            }

            #[inline]
            pub fn from_vec(data: Vec<$inner_type>) -> Self {
                Self(data)
            }

            #[inline]
            pub fn set(&mut self, index: impl Into<usize>, v: $inner_type) {
                let index: usize = index.into();
                self.0.set(index as u32, v);
            }

            #[inline]
            pub fn get_inner(&self) -> Vec<$inner_type> {
                self.0.clone()
            }

            #[inline]
            pub fn get(&self, index: impl Into<usize>) -> $inner_type {
                let index: usize = index.into();

                self.get_unchecked(index as u32)
            }
        }

        impl Deref for $name {
            type Target = Vec<$inner_type>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

sized_array!(SizedAddressArray, Address);
sized_array!(SizedU128Array, u128);
sized_array!(SizedDecimalsArray, u32);

impl SizedU128Array {
    #[inline]
    pub fn default_val<const N: usize>(env: &Env) -> Self {
        Self::from_array(env, [0; N])
    }

    #[inline]
    pub fn add(&mut self, index: impl Into<usize>, v: u128) {
        let index: usize = index.into();
        self.set(index, self.get(index) + v);
    }

    #[inline]
    pub fn sub(&mut self, index: impl Into<usize>, v: u128) {
        let index: usize = index.into();
        self.set(index, self.get(index) - v);
    }
}

impl Sub for SizedU128Array {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(self.len() == rhs.len(), "SizedU128Array: unexpected len");
        let mut v = self.clone();

        for (i, (l, r)) in self.iter().zip(rhs.iter()).enumerate() {
            v.set(i, l - r);
        }

        v
    }
}