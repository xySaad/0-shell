use std::fs;
use std::io::ErrorKind;

pub fn mkdir(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("mkdir: missing operand");
        return 1;
    }

    let mut counter: i32 = 0;

    for dir in args {
        if dir.len() > 255 {
            eprintln!(
                "mkdir: cannot create directory '{}': File name too long",
                dir
            );
            counter += 1;
            continue;
        }

        match fs::create_dir(dir) {
            Ok(_) => {}
            Err(e) => {
                let msg = match e.kind() {
                    ErrorKind::AlreadyExists => "File exists",
                    ErrorKind::PermissionDenied => "Permission denied",
                    ErrorKind::NotADirectory => "Not a directory",
                    ErrorKind::NotFound => "No such file or directory",
                    ErrorKind::WouldBlock => "Operation would block",
                    ErrorKind::Interrupted => "Operation interrupted",
                    ErrorKind::Other => "Unknown error",
                    _ => "An error occurred",
                };
                eprintln!("mkdir: cannot create directory '{}': {}", dir, msg);
                counter += 1;
            }
        }
    }

    if counter > 0 { 1 } else { 0 }
}
