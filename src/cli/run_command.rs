use std::{
    io::{self, Read, Write},
    os::fd::AsRawFd,
    process::exit,
    thread::{JoinHandle, spawn},
};

use libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, c_int, close, dup2, fork, waitpid};

use crate::{cmd::*, compiler::interpreter::Command};

pub fn redirect(
    // stdin: Vec<Box<dyn Send + Write>>,
    stdout: Vec<Box<dyn Send + Write>>,
    stderr: Vec<Box<dyn Send + Write>>,
) -> Vec<JoinHandle<()>> {
    let mut handlers = Vec::new();
    for (mut redirections, io_stream) in [
        // (stdin, STDIN_FILENO),
        (stdout, STDOUT_FILENO),
        (stderr, STDERR_FILENO),
    ] {
        if redirections.len() == 0 {
            continue;
        }

        let (mut reader, writer) = match io::pipe() {
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
    return handlers;
}

// forks a command and returns exit status
pub fn run_command(cmd: Command) -> i32 {
    let Command {
        name,
        ref args,
        // stdin,
        stdout,
        stderr,
    } = cmd;

    let pid = unsafe { fork() };

    if pid == -1 {
        return 1;
    }

    // child
    if pid == 0 {
        let handlers = redirect(/*stdin, */ stdout, stderr);
        let exit_status = match &*name {
            "echo" => echo(args),
            "cp" => cp::cp(args),
            "exit" => exit(0),
            _ => {
                println!("Command '{}' not found", name);
                return 127;
            }
        };
        unsafe {
            close(STDIN_FILENO);
            close(STDOUT_FILENO);
            close(STDERR_FILENO);
        }
        for h in handlers {
            let _ = h.join();
        }
        exit(exit_status);
    };

    let status = 0 as *mut c_int;
    let wid = unsafe { waitpid(pid, status, 0) };
    if wid != pid {
        return 1;
    }
    return status as i32;
}
