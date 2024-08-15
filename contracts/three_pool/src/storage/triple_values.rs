use soroban_sdk::{contracttype, Address};

use core::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use super::common::Token;

#[macro_export]
macro_rules! triple_value {
    ($name:ident, $inner_type:ident) => {
        #[contracttype]
        #[derive(Debug, Clone)]
        pub struct $name {
            pub data: ($inner_type, $inner_type, $inner_type),
        }

        impl $name {
            pub fn to_array(&self) -> [$inner_type; 3] {
                [
                    self.data.0.clone(),
                    self.data.1.clone(),
                    self.data.2.clone(),
                ]
            }
        }

        impl Index<usize> for $name {
            type Output = $inner_type;

            fn index(&self, index: usize) -> &Self::Output {
                match index {
                    0 => &self.data.0,
                    1 => &self.data.1,
                    2 => &self.data.2,
                    _ => panic!("Unexpected index"),
                }
            }
        }

        impl IndexMut<usize> for $name {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                match index {
                    0 => &mut self.data.0,
                    1 => &mut self.data.1,
                    2 => &mut self.data.2,
                    _ => panic!("Unexpected index"),
                }
            }
        }

        impl Index<Token> for $name {
            type Output = $inner_type;

            fn index(&self, index: Token) -> &Self::Output {
                &self[index as usize]
            }
        }

        impl IndexMut<Token> for $name {
            fn index_mut(&mut self, index: Token) -> &mut Self::Output {
                &mut self[index as usize]
            }
        }

        impl From<[$inner_type; 3]> for $name {
            #[inline]
            fn from(value: [$inner_type; 3]) -> Self {
                Self {
                    data: (value[0].clone(), value[1].clone(), value[2].clone()),
                }
            }
        }

        impl From<($inner_type, $inner_type, $inner_type)> for $name {
            #[inline]
            fn from(data: ($inner_type, $inner_type, $inner_type)) -> Self {
                Self { data }
            }
        }
    };
}

triple_value!(TripleAddress, Address);
triple_value!(TripleU128, u128);
triple_value!(TripleU32, u32);

#[allow(clippy::derivable_impls)]
impl Default for TripleU128 {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl TripleU128 {
    #[inline]
    pub fn sum(&self) -> u128 {
        self.data.0 + self.data.1 + self.data.2
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.sum() == 0
    }
}
