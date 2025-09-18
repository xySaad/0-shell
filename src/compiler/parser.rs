use crate::compiler::{
    nodes::{self, Node, Sequence, SubstitutionKind},
    tokenizer::Tokenizer,
    tokens::{self, Token},
};

/// Parses shell input according to [`POSIX Shell Command Language`](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
pub struct Parser<T: Fn() -> String> {
    tokenizer: Tokenizer,
    context: Option<Token>,
    reader: Option<T>,
}

impl<T: Fn() -> String> Parser<T> {
    pub fn new(source: &str) -> Parser<T> {
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

    pub fn feed(&mut self, source: &str) {
        self.tokenizer.feed(source)
    }

    fn get_raw<P: Into<String>>(&mut self, first: P) -> String {
        let mut str = String::from(first.into());

        while let Token::RawChar(ch) = self.tokenizer.current {
            self.tokenizer.next();
            str.push(ch);
        }

        str
    }
    /// collects next tokens until `func` is true
    fn get_sequence<P: Fn(Token) -> bool>(&mut self, func: P) -> Sequence {
        let mut seq = Sequence::new();

        while func(self.tokenizer.current) {
            let next = self.next();

            // the whole input string has consumed but closing didn't occur yet
            if next.is_none() {
                match &self.reader {
                    Some(read) => {
                        self.feed(&read());
                    }
                    None => todo!("return error"),
                }

                continue;
            }

            seq.push(next.unwrap());
        }

        self.tokenizer.next(); // consume closing
        return seq;
    }

    fn get_quoted(&mut self, quote: tokens::Quote) -> Node {
        let inside_double = matches!(self.context, Some(Token::Quote(tokens::Quote::Double)));
        match quote {
            // single quote has no meaning inside double quotes
            tokens::Quote::Single if inside_double => Node::Raw(self.get_raw('\'')),
            q => {
                self.context = Some(Token::Quote(q));
                let seq = self.get_sequence(|t| !matches!(t, Token::Quote(q2) if quote == q2));
                match q {
                    tokens::Quote::Double => Node::Quoted {
                        kind: nodes::Quote::Double,
                        value: seq,
                    },
                    tokens::Quote::Back => Node::Substitution {
                        kind: SubstitutionKind::BackQuote,
                        value: seq,
                    },
                    tokens::Quote::Single => Node::Quoted {
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
        if matches!(self.tokenizer.current, Token::Bracket('(')) {
            // TODO: handle arithmetic expansion [https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_04]
            self.tokenizer.next(); // consume opening bracket '('
            return Node::Substitution {
                kind: SubstitutionKind::RoundBracket,
                value: self.get_sequence(|t| !matches!(t, Token::Bracket(')'))),
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
        use tokens::{Quote::*, Token::*};
        // Get the next token after the backslash
        let next = self.tokenizer.next();

        match next {
            None => match &self.reader {
                Some(read) => {
                    self.feed(&read());
                    self.escape_next()
                }
                None => todo!("return error"),
            },
            Some(next) => match self.context {
                Some(Quote(ctx)) if matches!(ctx, Double | Back) => {
                    match next {
                        // Inside double-quoted strings
                        DollarSign | Quote(Double | Back) | BackSlash if matches!(ctx, Double) => {
                            Node::Raw(next.into())
                        }
                        // Inside backquoted substitutions
                        DollarSign | Quote(Back) | BackSlash => Node::Raw(next.into()),
                        _ => {
                            // For non-escapable characters, preserve the backslash and include the character
                            let mut result = String::from("\\");
                            result.push_str(&String::from(next));
                            Node::Raw(result)
                        }
                    }
                }
                _ => Node::Raw(next.into()),
            },
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
            Token::RawChar(ch) => Node::Raw(self.get_raw(ch)),
            Token::Quote(q) => self.get_quoted(q),
            Token::DollarSign => self.handle_dollar_sign(),
            Token::BackSlash => self.escape_next(),
            Token::Operator(op) => Node::Operator(op),
            Token::Delimiter(d) => Node::Delimiter(d),
            Token::Bracket(ch) => Node::Raw(ch.into()),
            Token::EOF => Node::EOF,
        };

        self.context = parent; // in case the context has changed inside get_quote
        Some(node)
    }
}
