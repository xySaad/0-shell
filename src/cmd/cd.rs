use std::env::{set_current_dir};
use std::path::{Path, PathBuf};
use crate::cli::Shell;

// to do //
// should handle error output by message error !!
// should handle "////"

pub fn cd(args: &[String], shell: &mut Shell) -> i32 {
    // println!("befor: {:?}", current_dir());

    if args.len() > 1 {
        println!("0-shell: cd: too many arguments");
        return 1;
    }

    let current_pwd = shell.env.get("PWD").cloned().unwrap_or("/".to_string());

    let target = if args.is_empty() || args[0] == "--" {
        "~".to_string()
    } else {
        args[0].clone()
    };

    match target.as_str() {
        "~" => {
            let home  = shell.env.get("HOME").cloned().unwrap_or("/".to_string());

            match change_dir(&home, &current_pwd, shell) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }

        "-" => {
            let oldpwd = shell.env.get("OLDPWD").cloned().unwrap_or(current_pwd.clone());

            match change_dir(&oldpwd, &current_pwd, shell) {
                Ok(_) => {
                    println!("{}", oldpwd);
                },
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }

        ".." => {
            let mut logical = PathBuf::from(&current_pwd);
            logical.pop(); // remove last component logically (don’t resolve symlink)
            let new_path = logical.display().to_string();
            match change_dir(&new_path, &current_pwd, shell) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }

        other => {
            let pwd = Path::new(&current_pwd);
            let abs_path = if Path::new(other).is_absolute() {
                PathBuf::from(other)
            } else {
                pwd.join(other)
            };
            match change_dir(abs_path.to_str().unwrap(), &current_pwd, shell) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
            };
        }
    };

    // println!("after: {:?}", current_dir());
    0
}

fn change_dir(target: &str, oldpwd: &str, shell: &mut Shell) -> Result<(), String> {
    let path = Path::new(target);
    if !path.exists() {
        return Err(format!("cd: {}: No such directory", target));
    }
    println!("{:?}", path);
    match set_current_dir(path) {
        Ok(_) => {
            println!("Succes");
        },
        Err(e) => {
            println!("Error: {}", e);
            return Err("eeeeeeeeeeeeeeee".to_string());
        }
    };

    shell.env.set("OLDPWD", oldpwd);
    shell.env.set("PWD", &normalize_path(&PathBuf::from(target)));

    Ok(())
}

fn normalize_path(path: &Path) -> String {
    let mut parts: Vec<&str> = vec![];

    for comp in path.components() {
        match comp.as_os_str().to_str() {
            Some(".") => continue,
            Some("..") => {
                parts.pop();
            }
            Some(s) => parts.push(s),
            None => continue,
        }
    }

    let mut res = if path.is_absolute() {
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
    if start > 1 {
        res = res[start..].to_string();
    }
    res
}


// // path = "".to_string(); // handle
// // path = "~".to_string(); // handle in parser +++
// // path = "/".to_string(); // simple path +++
// // path = "..".to_string(); // handle ////
// // path = ".".to_string(); // do nothing
// // path = "-".to_string(); // enverments variable ///////
// // path = "my home".to_string(); // simple path //
// // path = "home/src".to_string(); // simple path //
// // path = "file.txt".to_string();
// // path = ".home".to_string(); // hiden file like folder
// // path = "/root".to_string();
// // path = "/home".to_string();
// // path = "~/home".to_string(); // handle in parser +++
// // path = "./home".to_string();
// // path = "../home".to_string();
// // path = "../../home".to_string();