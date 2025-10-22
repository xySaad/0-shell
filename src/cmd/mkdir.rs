use crate::utils::error::strerror;
use std::fs;

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
                let msg = if let Some(errno) = e.raw_os_error() {
                    strerror(errno)
                } else {
                    e.to_string()
                };

                eprintln!("mkdir: cannot create directory '{}': {}", dir, msg);
                counter += 1;
            }
        }
    }

    if counter > 0 { 1 } else { 0 }
}
