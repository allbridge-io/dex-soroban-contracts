use soroban_sdk::contracttype;

#[contracttype]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum Token {
    A = 0,
    B = 1,
    C = 2,
}

impl From<usize> for Token {
    fn from(value: usize) -> Self {
        match value {
            0 => Token::A,
            1 => Token::B,
            2 => Token::C,
            _ => unreachable!(),
        }
    }
}

impl Into<usize> for Token {
    fn into(self) -> usize {
        self as usize
    }
}

impl Token {
    pub fn third(&self, second: Token) -> Token {
        match (self, second) {
            (Token::A, Token::B) => Token::C,
            (Token::A, Token::C) => Token::B,
            (Token::B, Token::A) => Token::C,
            (Token::B, Token::C) => Token::A,
            (Token::C, Token::A) => Token::B,
            (Token::C, Token::B) => Token::A,
            _ => panic!("The same token"),
        }
    }
}
