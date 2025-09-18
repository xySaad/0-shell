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

    pub fn parse_line(&self, input: &str) -> (String, Vec<String>) {
        let p = Parser::with_reader(&input, &self.reader);
        return self.parse_sequence(p);
    }

    pub fn parse_sequence(&self, mut seq: impl Iterator<Item = Node>) -> (String, Vec<String>) {
        let mut cmd = String::new();
        while let Some(n) = seq.next() {
            if let Node::Delimiter(_) = n {
                if cmd.is_empty() {
                    continue;
                }
                break;
            }
            cmd.push_str(&self.node_to_string(n));
        }

        let mut args = Vec::new();
        while let Some(n) = seq.next() {
            if let Node::Delimiter(_) = n {
                continue;
            }
            args.push(self.node_to_string(n));
        }
        return (cmd, args);
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
            Node::Substitution { value, .. } => {
                let (cmd, args) = self.parse_sequence(value.into_iter());
                match run_command(&cmd, &args) {
                    Ok(res) => res,
                    Err(err) => {
                        let _ = cli::error(&err.to_string());
                        String::new()
                    }
                }
            }
            Node::Delimiter(ch) => ch.into(),
            Node::Operator(_op) => todo!(),
            Node::EOF => todo!(),
        }
    }
}
