use soroban_sdk::contracttype;

#[contracttype]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum Token {
    A = 0,
    B = 1,
}

impl From<usize> for Token {
    fn from(value: usize) -> Self {
        match value {
            0 => Token::A,
            1 => Token::B,
            _ => unreachable!(),
        }
    }
}

impl Token {
    pub fn opposite(&self) -> Token {
        match self {
            Token::A => Token::B,
            Token::B => Token::A,
        }
    }
}
