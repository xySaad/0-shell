use super::tokens::*;

#[derive(Debug, PartialEq)]
pub enum SubstitutionKind {
    RoundBracket,
    BackQuote,
}
pub type Sequence = Vec<Node>;

#[derive(Debug)]
pub enum Quote {
    Single,
    Double,
}

impl Into<char> for Quote {
    fn into(self) -> char {
        match self {
            Quote::Single => '\'',
            Quote::Double => '"',
        }
    }
}
/// TODO: use ignored `Quoted.kind` and `Substitution.kind`
pub enum Node {
    Raw(String),
    Quoted {
        #[allow(dead_code)]
        kind: Quote,
        value: Sequence,
    },
    ParameterExpansion(String),
    Substitution {
        #[allow(dead_code)]
        kind: SubstitutionKind,
        value: Sequence,
    },
    WhiteSpace(char),
    Operator(Operator),
    Delimiter,
    EOF,
}
