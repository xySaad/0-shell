use std::{env, fs, path::PathBuf};
use crate::utils::error::clear_error;

pub fn rm(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return 1;
    }

    let recursive = args.contains(&("-r").to_string());
    let paths: Vec<&String> = args.iter().filter(|iteam| *iteam != "-r").collect();

    if paths.is_empty() {
        eprintln!("rm: missing operand");
        return 1;
    }

    for arg in paths {
        let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push(arg);

        // Check if path exist
        if path.symlink_metadata().is_err() {
            eprintln!("rm: cannot remove '{}': No such file or directory", arg);
            continue;
        }

        let metadata = match path.symlink_metadata() {
            Ok(m) => m,
            Err(err) => {
                eprintln!("rm: cannot access '{}': {}", arg, clear_error(err));
                continue;
            }
        };

        // Directory
        if metadata.is_dir() {
            if recursive {
                if let Err(err) = fs::remove_dir_all(&path) {
                    eprintln!("rm: cannot remove '{}': {}", arg, clear_error(err));
                }
            }

            else { eprintln!("rm: cannot remove '{}': Is a directory", arg); }
        }

        // File
        else {
            if let Err(err) = fs::remove_file(&path) {
                eprintln!("rm: cannot remove '{}': {}", arg, clear_error(err));
            }
        }
    }

    0
}

// # Delete a single file
// rm file.txt

// # Delete multiple files
// rm file1.txt file2.txt file3.txt

// # Delete files with wildcard patterns
// rm *.txt          # All .txt files in current directory
// rm file*          # All files starting with "file"
// rm *.log *.tmp    # All .log and .tmp files

// # Delete a directory and all its contents (recursive)
// rm -r folder_name

// # Delete multiple directories
// rm -r dir1 dir2 dir3

// # Remove all files except specific ones
// rm !(important.txt)  # Requires extglob option

// # Remove all files in current directory (dangerous!)
// rm *

// # Delete hidden files
// rm .*
// rm .hidden_file

// # Remove files with special characters
// rm "file with spaces.txt"
// rm 'file-name.txt'

// # 2. Double-check with ls before rm
// ls *.txt
// rm *.txt