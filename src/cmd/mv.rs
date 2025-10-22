use std::fs;
use std::path::Path;

fn move_one(source: &Path, dest: &Path) -> bool {
    if !source.exists() {
        eprintln!("mv: cannot stat '{}': No such file or directory", source.display());
        return false;
    }

    if dest.exists() && dest.is_dir() {
        eprintln!("mv: cannot overwrite directory '{}' with non-directory", dest.display());
        return false;
    }

    match fs::rename(source, dest) {
        Ok(_) => true,
        Err(e) => {
            eprintln!(
                "mv: cannot move '{}' to '{}': {}",
                source.display(),
                dest.display(),
                e
            );
            false
        }
    }
}

pub fn mv(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("mv: missing file operand");
        return 1;
    }
    if args.len() == 1 {
        eprintln!("mv: missing destination file operand after '{}'", args[0]);
        return 1;
    }

    let dest = Path::new(args.last().unwrap());
    let sources = &args[..args.len() - 1];
    let mut all_ok = true;

    // Case 1: one source, dest not necessarily a directory
    if sources.len() == 1 && !dest.is_dir() {
        let source = Path::new(&sources[0]);
        return if move_one(source, dest) { 0 } else { 1 };
    }

    // Case 2: multiple sources (or dest is dir)
    if !dest.is_dir() {
        eprintln!("mv: target '{}' is not a directory", dest.display());
        return 1;
    }

    for src_str in sources {
        let source = Path::new(src_str);
        let Some(name) = source.file_name() else {
            eprintln!("mv: invalid source name '{}'", source.display());
            all_ok = false;
            continue;
        };
        let dest_file = dest.join(name);
        if !move_one(source, &dest_file) {
            all_ok = false;
        }
    }

    if all_ok { 0 } else { 1 }
}
