use crate::{
    cli::{self, run_command},
    compiler::interpreter::Interpreter,
};
use core::error;
use std::{
    io::{Write, stderr, stdin},
    process::exit,
};

pub fn read_line() -> String {
    let mut input = String::new();
    if let Err(e) = stdin().read_line(&mut input) {
        let _ = writeln!(stderr(), "{e}");
        exit(0)
    }
    if input.is_empty() {
        exit(0)
    }

    return input;
}

pub fn read_input() {
    let inter = Interpreter::new(read_line, run_command);

    loop {
        let input = read_line();
        if input.trim().is_empty() {
            let _ = cli::print("$ ");
            continue;
        }

        let commands = inter.parse_line(&input);
        for command in commands {
            if let Some(error) = command.error {
                eprint!("{error}");
                continue;
            }
            if command.name.trim().is_empty() {
                continue;
            }
            let exit_status = run_command(command);
        }
        let _ = cli::print("$ ");
    }
}
