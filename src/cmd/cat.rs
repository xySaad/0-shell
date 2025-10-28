use crate::utils::error::clear_error;
use std::fs;
use std::io::{self, BufRead, Write};

// Reads from standard input line by line and writes it to standard output
fn read_input() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle_out = stdout.lock();

    for line_result in stdin.lock().lines() {
        match line_result {
            Ok(line) => {
                if let Err(e) = writeln!(handle_out, "{}", line) {
                    eprintln!("cat: write error: {}", e);
                    break;
                }
                if let Err(e) = handle_out.flush() {
                    eprintln!("cat: flush error: {}", e);
                }
            }
            Err(e) => {
                if let Err(write_err) = writeln!(handle_out, "cat: stdin: {}", e) {
                    eprintln!("cat: failed to write error message: {}", write_err);
                }
                break;
            }
        }
    }
}

// Implements the behavior of the `cat` command.
pub fn cat(args: &[String]) -> i32 {
    let mut all_ok = true;

    if args.is_empty() {
        read_input();
        return 0;
    }

    for (_, filename) in args.iter().enumerate() {
        if filename == "-" {
            read_input();
        } else {
            match fs::read_to_string(filename) {
                Ok(content) => {
                    print!("{}", content);

                    if let Err(e) = io::stdout().flush() {
                        eprintln!("cat: {}: flush error: {}", filename, e);
                        all_ok = false;
                    }
                }
                Err(e) => {
                    eprintln!("cat: {}: {}", filename, clear_error(e));
                    all_ok = false
                }
            }
        }
    }

    if all_ok { 0 } else { 1 }
}
