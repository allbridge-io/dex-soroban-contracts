use soroban_sdk::contracttype;

#[contracttype]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum TwoToken {
    A = 0,
    B = 1,
}

impl From<usize> for TwoToken {
    fn from(value: usize) -> Self {
        match value {
            0 => TwoToken::A,
            1 => TwoToken::B,
            _ => unreachable!(),
        }
    }
}

impl From<TwoToken> for usize {
    fn from(value: TwoToken) -> Self {
        value as usize
    }
}

impl TwoToken {
    pub fn opposite(&self) -> TwoToken {
        match self {
            TwoToken::A => TwoToken::B,
            TwoToken::B => TwoToken::A,
        }
    }
}
