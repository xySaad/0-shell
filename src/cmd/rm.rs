use std::{env, fs, path::PathBuf};
use crate::utils::error::clear_error;

pub fn rm(mut args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return 1;
    }

    let mut recursive = false;
    let mut paths: Vec<String> = Vec::new();
    let limiter_idx = args.iter().position(|val| val == &("--").to_string());

    if let Some(idx) = limiter_idx {
        paths = args[idx + 1..].to_vec();
        args = &args[..idx];
    }

    for opperand in args {
        match opperand.as_str() {
            "-r" | "--r" => recursive = true,
            "---" => (),
            _ => {
                if opperand != "-" && opperand.starts_with("-") {
                    for char in opperand[1..].chars() {
                        if char == '-' {
                            eprintln!("rm: unrecognized option '{}'", opperand);
                            return 1;
                        } else if char != 'r'  {
                            eprintln!("rm: invalid option -- '{}'", char);
                            return 1;
                        } else {
                            recursive = true;
                        }
                    }
                }
                else { paths.push(opperand.to_string()) }
            }
        };
    }

    if paths.is_empty() {
        eprintln!("rm: missing operand");
        return 1;
    }

    for arg in paths {
        let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push(&arg);

        match arg.as_str() {
            "." | ".." | "/" => {
                eprintln!("rm: refusing to remove '{}' directory", arg);
                continue;
            },
            _ => {
                if arg.ends_with("/.") || arg.ends_with("/..") || arg.ends_with("/./") || arg.ends_with("/../") {
                    eprintln!("rm: refusing to remove '{}' directory", arg);
                    continue;
                }
            }
        };

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