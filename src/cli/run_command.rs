use std::process;

use crate::cmd::*;

pub fn run_command(cmd: &str, args: &[String]) -> Result<String, String> {
    match cmd {
        "echo" => echo(args),
        "cp" => cp::cp(args),
        "cd" => cd::cd(args),
        "rm" => rm::rm(args),
        "pwd" => pwd::pwd(args),
        "exit" => process::exit(0),
        "clear" => Ok("\x1b[H\x1b[2J\x1b[3J".into()),
        _ => Err(format!("Command '{}' not found", cmd)),
    }
}
