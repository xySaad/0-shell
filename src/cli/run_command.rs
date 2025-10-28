use std::process::exit;

use libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, c_int, close, fork, waitpid};

use crate::cmd::{clear::clear, *};
use crate::compiler::command::Command;

// forks a command and returns exit status
pub fn run_command(cmd: Command) -> i32 {
    let Command {
        name,
        ref args,
        io_streams,
        ..
    } = cmd;
    
    // let mut exit_status =
    match name.as_str() {
        "cd" => return cd::cd(args),
        "pwd" => return pwd::pwd(args),
        "clear" => return clear(),
        "exit" => exit(0),
        _ => {}
    }

    // if exit_status != 0 {
    //     exit(exit_status);
    // }

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
            "ls"=> ls::run_ls(args),
            "mkdir" => mkdir::mkdir(args),
            "rm" => rm::rm(args),
            "cat" => cat::cat(args),
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
