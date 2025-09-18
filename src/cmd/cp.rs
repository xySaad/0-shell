use std::fs;
/// cp utility according to https://pubs.opengroup.org/onlinepubs/9699919799/utilities/cp.html
/// TODO: handle allowed options
pub fn cp(args: &[String]) -> Result<String, String> {
    match args.len() {
        0 => Err("cp: missing file operand".into()),
        1 => Err(format!(
            "cp: missing destination file operand after '{}'",
            args[0]
        )),
        _ => {
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
                Err(e) => return Err(format!("cp: '{target}' {e}")),
            };

            if !target_meta.is_dir() {
                return Err(format!("cp: target '{target}' is not a directory"));
            }

            let src_files = &args[..args.len() - 1];
            let mut err = Vec::new();
            for src in src_files {
                // target_file is the destination path named by the concatenation of target,
                // a single <slash> character if target did not end in a <slash>,
                // and the last component of source_file.

                let src_last = src.split("/").last().unwrap_or(&src);
                let target_file = target.trim_end_matches("/").to_string() + "/" + src_last;
                if let Err(e) = perform_copy(src, &target_file) {
                    err.push(e);
                };
            }

            if err.is_empty() {
                return Err(err.join("\n"));
            }

            return Ok("".into());
        }
    }
}

fn perform_copy(src: &str, target: &str) -> Result<String, String> {
    let src_meta = match fs::metadata(src) {
        Err(e) => return Err(format!("cp: '{src}' {e}")),
        Ok(m) => m,
    };

    if src_meta.is_dir() {
        return Err(format!("cp: -r not specified; omitting directory '{src}'",));
    }

    if let Err(e) = fs::copy(src, &target) {
        return Err(format!("cp: '{src}' -> '{target}' {e}"));
    }

    return Ok("".into());
}
