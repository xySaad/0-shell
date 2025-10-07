use libc::dup;

use crate::{
    cli::run_command,
    compiler::{
        nodes::Node,
        parser::Parser,
        tokens::{
            Operator::{self, *},
            RedirectionKind::*,
        },
    },
};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Write, pipe},
    os::fd::{AsRawFd, FromRawFd},
};

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    // pub stdin: Vec<Box<dyn Send + Write>>,
    pub stdout: Vec<Box<dyn Send + Write>>,
    pub stderr: Vec<Box<dyn Send + Write>>,
}
impl Command {
    fn handle_operation(&mut self, op: Operator, opperand: String) {
        match op {
            SemiColon => todo!(),
            And => todo!(),
            AndIf => todo!(),
            Pipe => todo!(),
            Or => todo!(),
            Redirection(r) => {
                let file_name = opperand;
                let file = match OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(file_name)
                {
                    Err(e) => {
                        eprintln!("{e}");
                        return;
                    }
                    Ok(f) => f,
                };

                match r {
                    // Input => self.stdin.push(Box::new(file)),
                    Output => self.stdout.push(Box::new(file)),
                    Error => self.stderr.push(Box::new(file)),
                    OutputError => {
                        unsafe {
                            let fd = dup(file.as_raw_fd());
                            self.stderr.push(Box::new(file));
                            self.stdout.push(Box::new(File::from_raw_fd(fd)));
                        };
                    }
                };
            }
        }
    }
}
impl Default for Command {
    fn default() -> Self {
        Self {
            name: String::new(),
            args: Vec::new(),
            // stdin: Vec::new(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}

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

    pub fn parse_line(&self, input: &str) -> Command {
        let p = Parser::with_reader(&input, &self.reader);
        return self.parse_sequence(p);
    }

    /// Parses a sequence until a delimiter occurs or `seq` has been fully consumed.
    ///
    /// Note: when a delimiter occurs the result is returned immediatly and the rest of `seq` is not necessary consumed.
    pub fn parse_sequence(&self, seq: impl Iterator<Item = Node>) -> Command {
        let mut command_sequence = Vec::new();
        let mut current = String::new();
        let mut seq = seq.peekable();

        let mut command = Command::default();
        while let Some(node) = seq.next() {
            // delimiters like ';' and '\n' outside
            if let Node::Delimiter | Node::Operator(SemiColon) = node {
                break;
            }

            if let Node::Operator(op) = node {
                //consume white spaces
                while let Some(Node::WhiteSpace(_)) = seq.peek() {
                    seq.next();
                }

                if let Some(opperand) = seq.next() {
                    command.handle_operation(op, self.node_to_string(opperand));
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
        command_sequence.push(current.to_string());
        command.name = command_sequence[0].clone();
        command.args = command_sequence[1..].to_vec();
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
                let exit_status = run_command(command);
                continue;
            }
            
            // unwrap pipe on success, and redirect command output to it
            let (mut r, w) = p.unwrap();
            command.stdout.push(Box::new(w));
            let exit_status = run_command(command);

            // read redirected output
            let mut line = String::new();
            let _ = r.read_to_string(&mut line);
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
            Node::Substitution { value, .. } => self.parse_substitution(value.into_iter()),
            Node::WhiteSpace(ch) => ch.into(),
            Node::Operator(op) => op.into(),
            Node::Delimiter => "".into(),
            Node::EOF => "\0".into(),
        }
    }
}
