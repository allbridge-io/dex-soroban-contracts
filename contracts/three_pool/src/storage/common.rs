use soroban_sdk::contracttype;

#[contracttype]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum ThreePoolToken {
    A = 0,
    B = 1,
    C = 2,
}

impl From<usize> for ThreePoolToken {
    fn from(value: usize) -> Self {
        match value {
            0 => ThreePoolToken::A,
            1 => ThreePoolToken::B,
            2 => ThreePoolToken::C,
            _ => unreachable!(),
        }
    }
}

impl Into<usize> for ThreePoolToken {
    fn into(self) -> usize {
        self as usize
    }
}

impl ThreePoolToken {
    pub fn third(&self, second: ThreePoolToken) -> ThreePoolToken {
        match (self, second) {
            (ThreePoolToken::A, ThreePoolToken::B) => ThreePoolToken::C,
            (ThreePoolToken::A, ThreePoolToken::C) => ThreePoolToken::B,
            (ThreePoolToken::B, ThreePoolToken::A) => ThreePoolToken::C,
            (ThreePoolToken::B, ThreePoolToken::C) => ThreePoolToken::A,
            (ThreePoolToken::C, ThreePoolToken::A) => ThreePoolToken::B,
            (ThreePoolToken::C, ThreePoolToken::B) => ThreePoolToken::A,
            _ => panic!("The same token"),
        }
    }
}
