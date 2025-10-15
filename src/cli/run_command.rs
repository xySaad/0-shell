use std::process::exit;

use libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, c_int, close, dup2, fork, waitpid};

use crate::{cmd::*, compiler::interpreter::Command};

// forks a command and returns exit status
pub fn run_command(cmd: Command) -> i32 {
    let Command {
        name,
        ref args,
        io_streams,
    } = cmd;

    let pid = unsafe { fork() };

    if pid == -1 {
        return 1;
    }

    // child
    if pid == 0 {
        let handlers = io_streams.redirect();
        let exit_status = match name.as_str() {
            "echo" => echo(args),
            "cp" => cp::cp(args),
            "mv" => mv::mv(args),
            "exit" => exit(0),
            _ => {
                eprintln!("Command '{}' not found", name);
                127
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
