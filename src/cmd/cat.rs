use std::fs;
use std::io::{self, Read};

pub fn cat(args: &[String]) -> i32 {
    let mut output = String::new();

    if args.is_empty() {
        // Read from stdin if no arguments are provided
        match io::stdin().read_to_string(&mut output) {
            Ok(_) => {
                print!("{}", output);
                0
            }
            Err(e) => {
                eprintln!("cat: cannot read from stdin: {}", e);
                1
            }
        }
    } else {
        for filename in args {
            match fs::read_to_string(filename) {
                Ok(content) => output.push_str(&content),
                Err(e) => {
                    eprintln!("cat: cannot open '{}': {}", filename, e);
                    return 1;
                }
            }
        }
        print!("{}", output);
        0
    }
}