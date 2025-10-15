use std::fs;
use std::path::Path;

pub fn mv(args: &[String]) -> i32 {
    // Check if exactly two arguments are provided (source and destination)
    if args.len() != 2 {
        eprintln!("Usage: mv <source> <destination>");
        return 1;
    }

    let dest = Path::new(&args[args.len() - 1]);
    
    for (i, arg) in args.iter().enumerate() {
        if i == args.len() - 1 {
            break;
        }

        let source = Path::new(&arg);
        // Check if source exists
        if !source.exists() {
            eprintln!("Error: Source '{}' does not exist", source.display());
            return 1;
        }

        // If destination exists and is a directory, move source inside it
        if dest.is_dir() {
            let dest_file = dest.join(source.file_name().unwrap_or_default());
            return match fs::rename(source, &dest_file) {
                Ok(()) => {
                    0
                }
                Err(e) => {
                    eprintln!(
                        "Error moving '{}' to '{}': {}",
                        source.display(),
                        dest_file.display(),
                        e
                    );
                    1
                }
            };
        }

        // If destination exists, check if it's a file (mv won't overwrite directories without force)
        if dest.exists() {
            if dest.is_dir() {
                eprintln!(
                    "Error: Destination '{}' is a directory; cannot overwrite without force",
                    dest.display()
                );
                return 1;
            }
            // Remove existing file at destination (mimicking mv's overwrite behavior)
            if let Err(e) = fs::remove_file(dest) {
                eprintln!(
                    "Error removing existing destination '{}': {}",
                    dest.display(),
                    e
                );
                return 1;
            }
        }

        // Perform the rename/move operation
        match fs::rename(source, dest) {
            Ok(()) => {
                println!("Moved '{}' to '{}'", source.display(), dest.display());
                return 0;
            }
            Err(e) => {
                eprintln!(
                    "Error moving '{}' to '{}': {}",
                    source.display(),
                    dest.display(),
                    e
                );
                return 1;
            }
        }
    }
    0
}
