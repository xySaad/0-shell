#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quote {
    Single,
    Double,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectionKind {
    Input,
    Output,
    Error,
    OutputError,
}

impl From<RedirectionKind> for &str {
    fn from(x: RedirectionKind) -> Self {
        use RedirectionKind::*;
        match x {
            Input => "<",
            Output => ">",
            Error => "2>",
            OutputError => "&>",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    SemiColon,
    And,
    AndIf,
    Pipe,
    Or,
    Redirection(RedirectionKind),
}

impl From<Operator> for String {
    fn from(op: Operator) -> Self {
        match op {
            Redirection(ch) => ch.into(),
            SemiColon => ";",
            And => "&",
            Pipe => "|",
            AndIf => "&&",
            Or => "||",
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token {
    WhiteSpace(char),
    RawChar(char),
    Quote(Quote),
    Bracket(char),
    Operator(Operator),
    BackSlash,
    DollarSign,
    Tilde,
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
            '<' => Operator(Redirection(RedirectionKind::Input)),
            '>' => Operator(Redirection(RedirectionKind::Output)),
            '\\' => BackSlash,
            '$' => DollarSign,
            ' ' | '\n' | '\t' => WhiteSpace(ch),
            '~' => Tilde,
            _ => RawChar(ch),
        }
    }
}

impl From<Token> for String {
    fn from(t: Token) -> Self {
        match t {
            Tilde => '~',
            WhiteSpace(ch) => ch,
            RawChar(ch) => ch,
            Token::Quote(quote) => match quote {
                Quote::Single => '\'',
                Quote::Double => '"',
                Quote::Back => '`',
            },
            Bracket(ch) => ch,
            Token::Operator(op) => return op.into(),
            BackSlash => '\\',
            DollarSign => '$',
            EOF => '\0',
        }
        .into()
    }
}
