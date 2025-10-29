use std::fs;
use crate::utils::error::{clear_error};


pub fn mkdir(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("mkdir: missing operand");
        return 1;
    }

    let mut counter: i32 = 0;

    for dir in args {
        match fs::create_dir(dir) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("mkdir: cannot create directory '{}': {}", dir, clear_error(e));
                counter += 1;
            }
        }
    }

    if counter > 0 { 1 } else { 0 }
}
