use crate::{
    cli::{self, run_command},
    compiler::interpreter::Interpreter,
};
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
    let inter = Interpreter::new(read_line);

    loop {
        let input = read_line();
        let result = inter.parse_line(&input);
        let (cmd, args) = (&result[0], &result[1..]);
        if cmd.trim().is_empty() {
            let _ = cli::print("$ ");
            continue;
        }

        let _ = match run_command(cmd, args) {
            Ok(out) => cli::print(&out),
            Err(err) => cli::error(&err),
        };
        let _ = cli::print("\n$ ");
    }
}
