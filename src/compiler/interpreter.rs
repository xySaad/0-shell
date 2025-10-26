use crate::compiler::{command::Command, nodes::Node, parser::Parser, tokens::Operator};
use std::{env, io::{Read, pipe}, iter::Peekable};

pub struct Interpreter<R: Fn() -> String, E: Fn(Command) -> i32> {
    reader: R,
    executor: E,
}

impl<R: Fn() -> String, E: Fn(Command) -> i32> Interpreter<R, E> {
    pub fn new(reader: R, executor: E) -> Self {
        Self { reader, executor }
    }

    pub fn exec(&self, command: Command) -> i32 {
        (self.executor)(command)
    }

    pub fn envar(&self, key: &str) -> String {
        env::var(key).unwrap_or_default()
    }

    pub fn parse_line(&self, input: &str) -> Vec<Command> {
        let p = Parser::with_reader(&input, &self.reader);
        let mut p = p.peekable();
        let mut commands = Vec::new();

        while p.peek().is_some() {
            commands.push(self.parse_sequence(&mut p));
        }
        return commands;
    }

    /// Parses a sequence until a delimiter occurs or `seq` has been fully consumed.
    ///
    /// Note: when a delimiter occurs the result is returned immediatly and the rest of `seq` is not necessary consumed.
    pub fn parse_sequence(&self, seq: &mut Peekable<impl Iterator<Item = Node>>) -> Command {
        let mut command_sequence = Vec::new();
        let mut current = String::new();

        let mut command = Command::default();
        while let Some(node) = seq.next() {
            // delimiters like ';' and '\n' outside
            if let Node::Delimiter | Node::Operator(Operator::SemiColon) = node {
                break;
            }

            if let Node::Operator(op) = node {
                //consume white spaces
                while let Some(Node::WhiteSpace(_)) = seq.peek() {
                    seq.next();
                }

                if let Some(_) = seq.peek() {
                    match op {
                        Operator::SemiColon => (),
                        Operator::And => todo!(),
                        Operator::AndIf => todo!(),
                        Operator::Pipe => todo!(),
                        Operator::Or => todo!(),
                        Operator::Redirection(r) => {
                            command.handle_redirection(r, self.node_to_string(seq.next().unwrap()));
                        }
                    }
                }
                //TODO: return parse error
                continue;
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
            command_sequence.push(current.to_string());
            current.clear();
        }

        // push last argument
        if !current.is_empty() {
            command_sequence.push(current.to_string());
        }

        if command_sequence.len() > 0 {
            command.name = command_sequence[0].clone();
        }

        if command_sequence.len() > 1 {
            command.args = command_sequence[1..].to_vec();
        }

        return command;
    }

    /// Parses substitution sequences, executes each command, and joins results with spaces.
    /// https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_03
    pub fn parse_substitution(&self, seq: impl Iterator<Item = Node>) -> String {
        let mut iter = seq.peekable();
        let mut result = Vec::new();

        while let Some(_) = iter.peek() {
            let mut command = self.parse_sequence(&mut iter);
            let p = pipe();
            if p.is_err() {
                let exit_status = self.exec(command);
                continue;
            }

            // unwrap pipe on success, and redirect command output to it
            let (mut r, w) = p.unwrap();
            command.io_streams.stdout.push(Box::new(w));
            let exit_status = self.exec(command);

            // read redirected output
            let mut line = String::new();
            let _ = r.read_to_string(&mut line);
            result.push(line.trim_end().to_string());
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
            Node::Substitution { value, .. } => self.parse_substitution(value.into_iter()),
            Node::WhiteSpace(ch) => ch.into(),
            Node::Operator(op) => op.into(),
            Node::Delimiter => "".into(),
            Node::EOF => "\0".into(),
        }
    }
}