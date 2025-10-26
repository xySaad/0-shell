use std::fs;
use std::io::{self, BufRead, Write};

// Reads from standard input line by line and writes it to standard output
fn read_input() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle_out = stdout.lock();

    for line in stdin.lock().lines() {
        match line {
            Ok(l) => {
                writeln!(handle_out, "{}", l).unwrap();
            }
            Err(e) => {
                writeln!(handle_out, "cat: stdin: {}", e).unwrap();
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
                Ok(content) => print!("{}\n", content),
                Err(e) => {
                    eprintln!("cat: {}: {}", filename, e);
                    all_ok = false
                }
            }
        }
    }

    if all_ok { 0 } else { 1 }
}
