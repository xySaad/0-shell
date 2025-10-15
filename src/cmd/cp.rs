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
        return perform_copy(src, target);
    }

    // second synopsis:
    // target is not a directory or other errors
    let target_meta = match fs::metadata(target) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("cp: '{target}' {e}");
            return 1;
        }
    };

    if !target_meta.is_dir() {
        eprintln!("cp: target '{target}' is not a directory");
        return 1;
    }

    let src_files = &args[..args.len() - 1];
    let mut exit_status = 0;
    for src in src_files {
        // target_file is the destination path named by the concatenation of target,
        // a single <slash> character if target did not end in a <slash>,
        // and the last component of source_file.

        let src_last = src.split("/").last().unwrap_or(&src);
        let target_file = target.trim_end_matches("/").to_string() + "/" + src_last;
        exit_status += perform_copy(src, &target_file);
    }

    return if exit_status > 0 { 1 } else { 0 };
}

fn perform_copy(src: &str, target: &str) -> i32 {
    let src_meta = match fs::metadata(src) {
        Err(e) => {
            eprintln!("cp: '{src}' {e}");
            return 1;
        }
        Ok(m) => m,
    };

    if src_meta.is_dir() {
        eprintln!("cp: -r not specified; omitting directory '{src}'",);
        return 1;
    }

    if let Err(e) = fs::copy(src, &target) {
        eprintln!("cp: '{src}' -> '{target}' {e}");
        return 1;
    }

    return 0;
}
