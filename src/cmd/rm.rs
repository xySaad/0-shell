use crate::utils::error::clear_error;
use std::{env, fs, path::PathBuf};

pub fn rm(mut args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return 1;
    }
    println!("{:?}", args);

    let mut recursive = false;
    let mut paths: Vec<String> = Vec::new();
    let end_opt = args.iter().position(|val| val == &("--").to_string());

    if let Some(idx) = end_opt {
        paths = args[idx + 1..].to_vec();
        args = &args[..idx];
    }

    for opperand in args {
        match opperand.as_str() {
            "-r" => recursive = true,
            option if option.starts_with("-") && option.len() > 1 => {
                eprintln!("rm: unrecognized option '{}'", option);
            }
            _ => paths.push(opperand.to_string()),
        }
    }

    // for (i, val) in args.iter().enumerate() {
    //     match end_opt {
    //         Some(index) => match index != i {
    //             true => {
    //                 match &(*val.to_string()) {
    //                     "-r" => recursive = true,
    //                     _ => {
    //                         if index < i {
    //                             if val != "-" && val.starts_with("-") {
    //                                 eprintln!("rm: unrecognized option '{}'", val);
    //                                 return 1;
    //                             } else {
    //                                 paths.push(val);
    //                             }
    //                         } else {
    //                             paths.push(val);
    //                         }
    //                     }
    //                 };
    //             }
    //             _ => (),
    //         },
    //         None => {
    //             match &(*val.to_string()) {
    //                 "-r" => recursive = true,
    //                 _ => {
    //                     if val != "-" && val.starts_with("-") {
    //                         eprintln!("rm: unrecognized option '{}'", val);
    //                         return 1;
    //                     } else {
    //                         paths.push(val);
    //                     }
    //                 }
    //             };
    //         }
    //     };
    // }

    if paths.is_empty() {
        eprintln!("rm: missing operand");
        return 1;
    }

    for arg in &paths {
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
            } else {
                eprintln!("rm: cannot remove '{}': Is a directory", arg);
            }
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

// # Remove all files except specific ones
// rm !(important.txt)  # Requires extglob option
