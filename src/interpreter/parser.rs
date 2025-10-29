use std::env;

use crate::interpreter::{
    nodes::{self, Node, Sequence, SubstitutionKind},
    tokenizer::Tokenizer,
    tokens::{self, Quote, Token},
};

/// Parses shell input according to [`POSIX Shell Command Language`](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
pub struct Parser<T: Fn() -> String> {
    tokenizer: Tokenizer,
    context: Option<Token>,
    reader: Option<T>,
}

impl<T: Fn() -> String> Parser<T> {
    pub fn _new(source: &str) -> Parser<T> {
        Self {
            tokenizer: Tokenizer::new(source),
            context: None,
            reader: None,
        }
    }

    pub fn with_reader(source: &str, reader: T) -> Parser<T> {
        Self {
            tokenizer: Tokenizer::new(source),
            context: None,
            reader: Some(reader),
        }
    }

    pub fn feed(&mut self) -> bool {
        match &self.reader {
            None => false,
            Some(read) => {
                self.tokenizer.feed(&read());
                true
            }
        }
    }

    fn get_raw<P: Into<String>>(&mut self, first: P) -> String {
        let mut str = String::from(first.into());

        while let Token::RawChar(ch) = self.tokenizer.current {
            self.tokenizer.next();
            str.push(ch);
        }

        str
    }

    /// collects next tokens while `until` is false
    fn get_sequence<P: Fn(Token) -> bool>(&mut self, until: P) -> Sequence {
        let mut seq = Sequence::new();

        while !until(self.tokenizer.current) {
            if let Some(token) = self.next() {
                seq.push(token);
                continue;
            }

            // the whole input string has consumed but closing didn't occur yet
            // if couldn't feed with more input break with EOF
            if !self.feed() {
                seq.push(Node::EOF);
                break;
            }
        }

        self.tokenizer.next(); // consume closing
        return seq;
    }

    fn get_quoted(&mut self, quote: tokens::Quote) -> Node {
        let inside_double = matches!(self.context, Some(Token::Quote(tokens::Quote::Double)));

        match quote {
            // single quote has no meaning inside double quotes
            Quote::Single if inside_double => Node::Raw(self.get_raw('\'')),
            q => {
                self.context = Some(Token::Quote(q));
                let seq = self.get_sequence(|t| matches!(t, Token::Quote(q2) if quote == q2));
                match q {
                    Quote::Double => Node::Quoted {
                        kind: nodes::Quote::Double,
                        value: seq,
                    },
                    Quote::Back => Node::Substitution {
                        kind: SubstitutionKind::BackQuote,
                        value: seq,
                    },
                    Quote::Single => Node::Quoted {
                        kind: nodes::Quote::Single,
                        value: seq,
                    },
                }
            }
        }
    }

    fn handle_dollar_sign(&mut self) -> Node {
        self.context = Some(Token::DollarSign);

        // handle Substitution $(...)
        if let Token::Bracket('(') = self.tokenizer.current {
            // TODO: handle arithmetic expansion [https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_04]
            self.tokenizer.next(); // consume opening bracket '('
            return Node::Substitution {
                kind: SubstitutionKind::RoundBracket,
                value: self.get_sequence(|t| matches!(t, Token::Bracket(')'))),
            };
        }

        // handle paramter expansion
        if let Token::RawChar(ch) = self.tokenizer.current {
            self.tokenizer.next(); //consume the raw character after $
            return Node::ParameterExpansion(self.get_raw(ch));
        }

        Node::Raw("$".into())
    }

    fn escape_next(&mut self) -> Node {
        use Token::*;
        use tokens::Quote::*;
        // Get the next token after the backslash
        loop {
            // the whole input string has consumed but closing didn't occur yet
            let Some(token) = self.tokenizer.next() else {
                if self.feed() {
                    continue;
                }

                // if couldn't feed with more input break with EOF
                return Node::EOF;
            };

            // https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_02_01
            if let WhiteSpace('\n') = token {
                return self.next().unwrap_or(Node::EOF);
            }

            return match self.context {
                // inside Double-Quotes https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_02_03
                Some(Quote(Double)) => match token {
                    DollarSign | Quote(Back | Double) | BackSlash => Node::Raw(token.into()),
                    _ => Node::Raw(String::from("\\") + &String::from(token)),
                },

                _ => Node::Raw(token.into()),
            };
        }
    }
    fn handle_white_space(&self, w: char) -> Node {
        use Token::*;
        use tokens::Quote::*;

        let inside_substitution = matches!(self.context, Some(DollarSign | Quote(Back)));
        // escaped newline in substitution is a line continuation
        if inside_substitution && w == '\n' {
            Node::Delimiter
        } else {
            Node::WhiteSpace(w)
        }
    }
}

impl<T: Fn() -> String> Iterator for Parser<T> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.tokenizer.next()?;

        // anything inside single quote is literal and has no meaning, it should be handled as raw
        if matches!(self.context, Some(Token::Quote(tokens::Quote::Single))) {
            return Some(Node::Raw(self.get_raw(current)));
        }

        let parent = self.context;
        let node = match current {
            Token::Tilde => {
                if self.context == Some(Token::Quote(Quote::Double)) {
                    Node::Raw("~".into())
                } else {
                    Node::Raw(env::var("HOME").unwrap_or_default())
                }
            }
            Token::RawChar(ch) => Node::Raw(self.get_raw(ch)),
            Token::Quote(q) => self.get_quoted(q),
            Token::DollarSign => self.handle_dollar_sign(),
            Token::BackSlash => self.escape_next(),
            Token::Operator(op) => Node::Operator(op),
            Token::WhiteSpace(w) => self.handle_white_space(w),
            Token::Bracket(ch) => Node::Raw(ch.into()),
            Token::EOF => Node::EOF,
        };

        self.context = parent; // in case the context has changed inside get_quote
        Some(node)
    }
}
