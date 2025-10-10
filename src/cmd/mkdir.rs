use std::fs;
use std::io::ErrorKind;

pub fn mkdir(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err(String::from("mkdir: missing operand"));
    }

    for dir in args {
        if let Err(e) = fs::create_dir(dir) {
            // I added this to avoid the printing of the whole error
            let msg = match e.kind() {
            ErrorKind::AlreadyExists => "File exists",
            ErrorKind::PermissionDenied => "Permission denied",
            ErrorKind::NotFound => "No such file or directory",
            ErrorKind::WouldBlock => "Operation would block",
            ErrorKind::Interrupted => "Operation interrupted",
            ErrorKind::Other => "Unknown error",
            _ => "An error occurred",
        };
            // here I returned the error with just the message
            return  Err(format!("mkdir: cannot create directory '{}': {}", dir, msg));
        }
    }
    Ok(String::new())
}