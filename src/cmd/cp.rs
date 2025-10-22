use crate::utils::error::StrError;
use std::fs;

/// minimal implementation of cp utility according to [POSIX: cp](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cp.html)
///
/// TODO: handle allowed options
pub fn cp(args: &[String]) -> i32 {
    if args.len() == 0 {
        eprintln!("cp: missing file operand");
        return 1;
    }

    if args.len() == 1 {
        eprintln!("cp: missing destination file operand after '{}'", args[0]);
        return 1;
    }

    let src = &args[0];
    let target = &args[args.len() - 1];

    // first synopsis:
    if args.len() == 2 {
        let target_file = match fs::metadata(target) {
            // second synopsis:
            Ok(m) if m.is_dir() => &join_target_path(src, target),
            _ => target,
        };

        return perform_copy(src, target_file);
    }

    // second synopsis:
    // target is not a directory or other errors
    match fs::metadata(target) {
        Ok(m) => {
            if !m.is_dir() {
                eprintln!("cp: target '{target}': Not a directory");
                return 1;
            }
        }
        Err(e) => {
            eprintln!("cp: target '{target}': {}", e.str());
            return 1;
        }
    };

    let src_files = &args[..args.len() - 1];
    let mut exit_status = 0;

    for src in src_files {
        let target_file = join_target_path(src, target);
        exit_status += perform_copy(src, &target_file);
    }

    return if exit_status > 0 { 1 } else { 0 };
}

/// the destination path named by the concatenation of `target`,
/// a single `/` character if `target` did not end in a `/`,
/// and the last component of `source_file`.
pub fn join_target_path(source_file: &str, target: &str) -> String {
    let src_last = source_file.split("/").last().unwrap_or(&source_file);
    let target_file = target.trim_end_matches("/").to_string() + "/" + src_last;
    return target_file;
}

fn perform_copy(src: &str, target: &str) -> i32 {
    let src_meta = match fs::metadata(src) {
        Err(e) => {
            eprintln!("cp: cannot stat '{src}' {}", e.str());
            return 1;
        }
        Ok(m) => m,
    };

    if src_meta.is_dir() {
        eprintln!("cp: -r not specified; omitting directory '{src}'",);
        return 1;
    }

    if let Ok(src_abs) = fs::canonicalize(src) {
        if let Ok(target_abs) = fs::canonicalize(target) {
            if src_abs == target_abs {
                eprintln!("cp: '{src}' and '{target}' are the same file");
                return 1;
            }
        }
    }
    if let Err(e) = fs::copy(src, target) {
        eprintln!("cp: target '{target}': {}", e.str());
        return 1;
    }

    return 0;
}
