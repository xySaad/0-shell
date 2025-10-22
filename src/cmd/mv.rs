use std::fs;
use std::path::Path;

pub fn mv(args: &[String]) -> i32 {
    // Check for at least two arguments (one source and one destination)
    if args.len() == 0 {
        eprintln!("mv: missing file operand");
        return 1;
    } else if args.len() == 1 {
        eprintln!("mv: missing destination file operand after '{}'", args[0]);
        return 1;
    }

    let dest = Path::new(&args[args.len() - 1]);
    let sources = &args[..args.len() - 1];
    let mut success = true;

    // If only one source, handle rename or move
    if sources.len() == 1 {
        let source = Path::new(&sources[0]);
        if !source.exists() {
            eprintln!("mv: cannot stat '{}': No such file or directory", source.display());
            return 1;
        }

        // If destination is a directory, move source inside it
        if dest.is_dir() {
            let dest_file = dest.join(source.file_name().unwrap_or_default());
            return match fs::rename(source, &dest_file) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("mv: cannot move '{}' to '{}': {}", source.display(), dest_file.display(), e);
                    1
                }
            };
        }

        // If destination exists, check if it's a file (won't overwrite directories)
        if dest.exists() {
            if dest.is_dir() {
                eprintln!("mv: cannot overwrite directory '{}' with non-directory", dest.display());
                return 1;
            }
            // Remove existing file at destination
            if let Err(e) = fs::remove_file(dest) {
                eprintln!("mv: cannot remove '{}': {}", dest.display(), e);
                return 1;
            }
        }

        // Perform rename/move
        return match fs::rename(source, dest) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("mv: cannot move '{}' to '{}': {}", source.display(), dest.display(), e);
                1
            }
        };
    }

    // For multiple sources, destination must be a directory
    if !dest.is_dir() {
        eprintln!("mv: target '{}' is not a directory", dest.display());
        return 1;
    }

    // Move each source to the destination directory
    for source_str in sources {
        let source = Path::new(source_str);
        if !source.exists() {
            eprintln!("mv: cannot stat '{}': No such file or directory", source.display());
            success = false;
            continue;
        }

        let dest_file = dest.join(source.file_name().unwrap_or_default());
        if let Err(e) = fs::rename(source, &dest_file) {
            eprintln!("mv: cannot move '{}' to '{}': {}", source.display(), dest_file.display(), e);
            success = false;
        }
    }

    if success { 0 } else { 1 }
}