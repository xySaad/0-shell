use crate::{
    compiler::tokens::RedirectionKind::{self, *},
    utils::error::StrError,
};
use libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, dup, dup2};
use std::{
    fs::{File, OpenOptions},
    io::{Error, Read, Write, pipe},
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
    pub error: Option<Error>,
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
    pub fn handle_redirection(&mut self, r: RedirectionKind, opperand: String) {
        let io_streams = &mut self.io_streams;

        let file_name = opperand;
        let mut opts = OpenOptions::new();
        if let Input = r {
            opts.read(true);
        } else {
            opts.create(true).write(true).truncate(true);
        }

        let file = opts.open(file_name);
        if let Err(e) = file {
            self.error = Some(Error::new(e.kind(), format!("shell: test*: {}\n", e.str())));
            return;
        }

        let file = file.unwrap();
        match r {
            Input => io_streams.stdin.push(Box::new(file)),
            Output => io_streams.stdout.push(Box::new(file)),
            RedirectionKind::Error => io_streams.stderr.push(Box::new(file)),
            OutputError => {
                unsafe {
                    let fd = dup(file.as_raw_fd());
                    io_streams.stderr.push(Box::new(file));
                    io_streams.stdout.push(Box::new(File::from_raw_fd(fd)));
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
            error: None,
        }
    }
}
