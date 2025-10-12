use std::fs;
use std::io::ErrorKind;

pub fn mkdir(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err(String::from("mkdir: missing operand"));
    }

    let mut errors: Vec<String> = Vec::new();

    for dir in args {
        match fs::create_dir(dir) {
            Ok(_) => {}
            Err(e) => {
                let msg = match e.kind() {
                    ErrorKind::AlreadyExists => "File exists",
                    ErrorKind::PermissionDenied => "Permission denied",
                    ErrorKind::NotFound => "No such file or directory",
                    ErrorKind::WouldBlock => "Operation would block",
                    ErrorKind::Interrupted => "Operation interrupted",
                    ErrorKind::Other => "Unknown error",
                    _ => "An error occurred",
                };
                errors.push(format!("mkdir: cannot create directory '{}': {}", dir, msg));
            }
        }
    }

    if !errors.is_empty() {
        let err = errors.join("\n");
        return Err(err);
    }
    Ok(String::new())
}
