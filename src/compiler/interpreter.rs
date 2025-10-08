use libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, dup, dup2};

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
    process::exit,
    thread::{JoinHandle, spawn},
};

pub struct IoStreams {
    pub stdin: Vec<Box<dyn Send + Read>>,
    pub stdout: Vec<Box<dyn Send + Write>>,
    pub stderr: Vec<Box<dyn Send + Write>>,
}

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub io_streams: IoStreams,
}
impl IoStreams {
    pub fn redirect(self) -> Vec<JoinHandle<()>> {
        let IoStreams {
            stdout,
            stderr,
            stdin,
        } = self;
        let mut handlers = Vec::new();
        for (mut redirections, io_stream) in [(stdout, STDOUT_FILENO), (stderr, STDERR_FILENO)] {
            if redirections.len() == 0 {
                continue;
            }

            let (mut reader, writer) = match pipe() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("pipe failed: {e}");
                    exit(1);
                }
            };

            if unsafe { dup2(writer.as_raw_fd(), io_stream) } == -1 {
                eprintln!("dup2 failed: {}", std::io::Error::last_os_error());
                exit(1);
            };

            let handler = spawn(move || {
                let mut buf = [0; 1024];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    for file in &mut redirections {
                        //todo write until n has been written
                        let _ = file.write(&buf[..n]);
                    }
                }
            });

            handlers.push(handler);
        }

        if stdin.len() == 0 {
            return handlers;
        }
        handlers.push(Self::redirect_in(stdin));

        return handlers;
    }
    pub fn redirect_in(mut stdin: Vec<Box<dyn Read + Send>>) -> JoinHandle<()> {
        let (reader, mut writer) = match pipe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("pipe failed: {e}");
                exit(1);
            }
        };

        if unsafe { dup2(reader.as_raw_fd(), STDIN_FILENO) } == -1 {
            eprintln!("dup2 failed: {}", std::io::Error::last_os_error());
            exit(1);
        };

        let handler = spawn(move || {
            let mut buf = [0; 1024];
            for file in &mut stdin {
                while let Ok(n) = file.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    //todo write until n has been written
                    let _ = writer.write(&buf[..n]);
                }
            }
        });

        return handler;
    }
}

impl Command {
    fn handle_operation(&mut self, op: Operator, opperand: String) {
        let io_streams = &mut self.io_streams;
        match op {
            SemiColon => todo!(),
            And => todo!(),
            AndIf => todo!(),
            Pipe => todo!(),
            Or => todo!(),
            Redirection(r) => {
                let file_name = opperand;
                let mut opts = OpenOptions::new();
                if let Input = r {
                    opts.read(true);
                } else {
                    opts.create(true).write(true).truncate(true);
                }

                let file = opts.open(file_name);
                if let Err(e) = file {
                    eprintln!("{e}");
                    return;
                }

                let file = file.unwrap();
                match r {
                    Input => io_streams.stdin.push(Box::new(file)),
                    Output => io_streams.stdout.push(Box::new(file)),
                    Error => io_streams.stderr.push(Box::new(file)),
                    OutputError => {
                        unsafe {
                            let fd = dup(file.as_raw_fd());
                            io_streams.stderr.push(Box::new(file));
                            io_streams.stdout.push(Box::new(File::from_raw_fd(fd)));
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
            io_streams: IoStreams {
                stdin: Vec::new(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
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
        if !current.is_empty() {
            command_sequence.push(current.to_string());
        }
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
            command.io_streams.stdout.push(Box::new(w));
            let exit_status = run_command(command);

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
