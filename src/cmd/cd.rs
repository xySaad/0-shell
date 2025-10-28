use std::env;
use std::path::{Path, PathBuf};
use crate::utils::error::clear_error;

pub fn cd(args: &[String]) -> i32 {
    if args.len() > 1 {
        println!("0-shell: cd: too many arguments");
        return 1;
    }

    let current_pwd = env::var("PWD").unwrap_or("/".to_string());
    let target = if args.is_empty() {
        "".to_string()
    } else {
        args[0].clone()
    };

    match target.as_str() {
        "-" => {
            let oldpwd = env::var("OLDPWD").unwrap_or(String::from(&current_pwd));

            match change_dir(&oldpwd, &current_pwd, "-") {
                Ok(_) => println!("{}", oldpwd),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }

        ".." => {
            let mut logical = PathBuf::from(&current_pwd);
            logical.pop();

            let new_path = logical.display().to_string();

            match change_dir(&new_path, &current_pwd, "..") {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }

        other => {

            // Case "~" similar
            if other == "" || other == "--" {
                let home  = env::var("HOME").unwrap_or(String::from("/"));

                match change_dir(&home, &current_pwd, other) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                        return 1;
                    }
                };
            }

            // Simple case
            else {
                let pwd = Path::new(&current_pwd);
                let abs_path = if Path::new(other).is_absolute() {
                    PathBuf::from(other)
                } else {
                    pwd.join(other)
                };

                match change_dir(&abs_path.display().to_string(), &current_pwd, other) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("{}", e);
                        return 1;
                    }
                };
            }
        }
    };

    0
}

fn change_dir(target: &str, oldpwd: &str, input: &str) -> Result<(), String> {
    let new_path = simple_path(&PathBuf::from(&target));
    let path = Path::new(&new_path);
    if !path.exists() {
        return Err(format!("0-shell: cd: {}: No such file or directory", input));
    }

    match env::set_current_dir(path) {
        Ok(_) => (),
        Err(e) => {
            return Err(format!("0-shell: cd: {}: {}", input, clear_error(e)));
        }
    };

    unsafe {
        env::set_var("OLDPWD", oldpwd);
        env::set_var("PWD", new_path);
    }

    Ok(())
}

fn simple_path(path: &Path) -> String {
    let mut parts: Vec<&str> = vec![];

    for comp in path.components() {
        match comp.as_os_str().to_str() {
            Some(".") => continue,
            Some("..") => { parts.pop(); }
            Some(s) => parts.push(s),
            None => continue,
        }
    }

    let res = if path.is_absolute() {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    };

    let mut start = 0;
    for (i, c) in res.clone().chars().enumerate() {
        if c == '/' {
            start = i;
            continue;
        }
        break
    }

    res[start..].to_string()
}