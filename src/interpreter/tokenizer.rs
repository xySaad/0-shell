use super::tokens::{
    Operator::*,
    Token::{self, *},
};

use std::{iter::Peekable, vec::IntoIter};

pub struct Tokenizer {
    pub chars: Peekable<IntoIter<char>>,
    pub current: Token,
}

impl Tokenizer {
    pub fn new(source: &str) -> Self {
        let mut chars = source.chars().collect::<Vec<char>>().into_iter().peekable();
        let current = chars.next().map_or(EOF, Into::into);

        Self { chars, current }
    }

    /// replaces the current source with a new one
    pub fn feed(&mut self, source: &str) {
        self.chars = source.chars().collect::<Vec<char>>().into_iter().peekable();
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let token = match (self.current, self.chars.peek()) {
            (Operator(And), Some('&')) => {
                self.chars.next();
                Operator(AndIf)
            }
            (Operator(Pipe), Some('|')) => {
                self.chars.next();
                Operator(Or)
            }
            (token, _) => token,
        };

        self.current = self.chars.next()?.into();
        Some(token)
    }
}
