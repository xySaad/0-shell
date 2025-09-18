#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quote {
    Single,
    Double,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    SemiColon,
    And,
    AndIf,
    Pipe,
    Or,
    ///### Not Implemented Yet
    Redirection(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    Delimiter(char),
    RawChar(char),
    Quote(Quote),
    Bracket(char),
    Operator(Operator),
    BackSlash,
    DollarSign,
    EOF,
}

use self::{Operator::*, Quote::*, Token::*};
impl From<char> for Token {
    fn from(ch: char) -> Self {
        match ch {
            '\'' => Quote(Single),
            '"' => Quote(Double),
            '`' => Quote(Back),
            '{' | '}' | '(' | ')' => Bracket(ch),
            ';' => Operator(SemiColon),
            '&' => Operator(And),
            '|' => Operator(Pipe),
            '<' | '>' => Operator(Redirection(ch)),
            '\\' => BackSlash,
            '$' => DollarSign,
            ' ' | '\n' | '\t' => Delimiter(ch),
            _ => RawChar(ch),
        }
    }
}

impl From<Token> for String {
    fn from(t: Token) -> Self {
        match t {
            Delimiter(ch) => ch,
            RawChar(ch) => ch,
            Token::Quote(quote) => match quote {
                Quote::Single => '\'',
                Quote::Double => '"',
                Quote::Back => '`',
            },
            Bracket(ch) => ch,
            Token::Operator(op) => match op {
                Redirection(ch) => ch,
                SemiColon => ';',
                And => '&',
                Pipe => '|',
                AndIf => return "&&".into(),
                Or => return "||".into(),
            }
            .into(),
            BackSlash => '\\',
            DollarSign => '$',
            EOF => '\0',
        }
        .into()
    }
}
