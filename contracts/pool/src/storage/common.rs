use soroban_sdk::contracttype;

#[contracttype]
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    A2B,
    B2A,
}

impl Direction {
    #[inline]
    pub fn get_tokens(&self) -> (Token, Token) {
        match self {
            Direction::A2B => (Token::A, Token::B),
            Direction::B2A => (Token::B, Token::A),
        }
    }
}

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
