use crate::{
    cli::{self, run_command},
    compiler::{nodes::Node, parser::Parser},
};
use std::env;

pub struct Interpreter<T: Fn() -> String> {
    reader: T,
}

impl<T: Fn() -> String> Interpreter<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    pub fn envar(&self, key: &str) -> String {
        env::var(key).unwrap_or_default()
    }

    pub fn parse_line(&self, input: &str) -> Vec<String> {
        let p = Parser::with_reader(&input, &self.reader);
        return self.parse_sequence(p);
    }

    /// Parses a sequence until a delimiter occurs or `seq` has been fully consumed.
    ///
    /// Note: when a delimiter occurs the result is returned immediatly and the rest of `seq` is not necessary consumed.
    pub fn parse_sequence(&self, mut seq: impl Iterator<Item = Node>) -> Vec<String> {
        let mut command = Vec::new();
        let mut current = String::new();

        while let Some(node) = seq.next() {
            // delimiters like ';' and '\n' outside
            if let Node::Delimiter = node {
                break;
            }

            // push non-whitespace characters
            if !matches!(node, Node::WhiteSpace(_)) {
                current.push_str(&self.node_to_string(node));
                continue;
            }

            // skip leading white spaces
            if current.is_empty() {
                continue;
            }

            // separate arguments by white spaces
            command.push(current.to_string());
            current.clear();
        }

        // push last argument
        command.push(current.to_string());
        return command;
    }

    /// Parses substitution sequences, executes each command, and joins results with spaces.
    pub fn parse_substitution(&self, seq: impl Iterator<Item = Node>) -> String {
        let mut iter = seq.peekable();
        let mut result = Vec::new();

        while let Some(_) = iter.peek() {
            let command = self.parse_sequence(&mut iter);
            let line = match run_command(&command[0], &command[1..]) {
                Ok(res) => res,
                Err(err) => {
                    let _ = cli::error(&err.to_string());
                    String::new()
                }
            };
            result.push(line);
        }

        return result.join(" ");
    }

    fn node_to_string(&self, node: Node) -> String {
        match node {
            Node::Raw(str) => str,
            Node::Quoted { value, .. } => {
                let mut res = String::new();

                for node in value {
                    res.push_str(&self.node_to_string(node));
                }
                return res;
            }
            Node::ParameterExpansion(param) => self.envar(&param),
            // https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_03
            Node::Substitution { value, .. } => self.parse_substitution(value.into_iter()),
            Node::WhiteSpace(ch) => ch.into(),
            Node::Operator(_op) => todo!(),
            // Delimiter shouldn't occur here!
            Node::Delimiter => "".into(),
            Node::EOF => todo!(),
        }
    }
}
