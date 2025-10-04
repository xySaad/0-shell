use std::process;

use crate::cmd::*;

pub fn run_command(cmd: &str, args: &[String]) -> Result<String, String> {
    match cmd {
        "echo" => echo(args),
        "cp" => cp::cp(args),
        "cd" => cd::cd(args),
        "rm" => rm::rm(args),
        "pwd" => pwd::pwd(args),
        "mkdir" => mkdir::mkdir(args),
        "exit" => process::exit(0),
        _ => Err(format!("Command '{}' not found", cmd)),
    }
}
