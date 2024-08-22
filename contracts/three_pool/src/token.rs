use soroban_sdk::contracttype;

#[contracttype]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum ThreeToken {
    A = 0,
    B = 1,
    C = 2,
}

impl From<usize> for ThreeToken {
    fn from(value: usize) -> Self {
        match value {
            0 => ThreeToken::A,
            1 => ThreeToken::B,
            2 => ThreeToken::C,
            _ => unreachable!(),
        }
    }
}

impl From<ThreeToken> for usize {
    fn from(value: ThreeToken) -> Self {
        value as usize
    }
}

impl ThreeToken {
    pub fn third(&self, second: ThreeToken) -> ThreeToken {
        match (self, second) {
            (ThreeToken::A, ThreeToken::B) => ThreeToken::C,
            (ThreeToken::A, ThreeToken::C) => ThreeToken::B,
            (ThreeToken::B, ThreeToken::A) => ThreeToken::C,
            (ThreeToken::B, ThreeToken::C) => ThreeToken::A,
            (ThreeToken::C, ThreeToken::A) => ThreeToken::B,
            (ThreeToken::C, ThreeToken::B) => ThreeToken::A,
            _ => panic!("The same token"),
        }
    }
}
