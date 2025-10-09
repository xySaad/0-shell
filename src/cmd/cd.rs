use std::env;
use std::path::{Path, PathBuf};

// to do //
// should handle error output by message error !!

pub fn cd(args: &[String]) -> Result<String, String> {
    println!("arguments: {:?}", args);
    println!("befor: {:?}", env::current_dir());

    if args.len() > 1 {
        return Ok("0-shell: cd: too many arguments".to_string());
    }

    let current_pwd = env::var("PWD").unwrap_or_else(|_| {
        env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| String::from("/"))
    });

    let target = if args.is_empty() || args[0] == "--" {
        "~".to_string()
    } else {
        args[0].clone()
    };

    match target.as_str() {
        "~" => {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("/"));
            change_dir(&home, &current_pwd)?;
        }

        "-" => {
            let oldpwd = env::var("OLDPWD").unwrap_or_else(|_| current_pwd.clone());
            change_dir(&oldpwd, &current_pwd)?;
            println!("{}", oldpwd);
        }

        ".." => {
            let mut logical = PathBuf::from(&current_pwd);
            logical.pop(); // remove last component logically (don’t resolve symlink)
            let new_path = logical.display().to_string();
            change_dir(&new_path, &current_pwd)?;
        }

        other => {
            let pwd = Path::new(&current_pwd);
            let abs_path = if Path::new(other).is_absolute() {
                PathBuf::from(other)
            } else {
                pwd.join(other)
            };
            change_dir(abs_path.to_str().unwrap(), &current_pwd)?;
        }
    };

    println!("after: {:?}", env::current_dir());
    Ok("OK".to_string())

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
}

fn change_dir(target: &str, oldpwd: &str) -> Result<(), String> {
    let path = Path::new(target);
    if !path.exists() {
        return Err(format!("cd: {}: No such directory", target));
    }
    //  else {
    //     Path::new("/home/amellagu/.local/share/Trash/files/")
    // }

    if let Err(e) = env::set_current_dir(path) {
        return Err(format!("cd: {}: {}", target, e));
    }

    unsafe { env::set_var("OLDPWD", oldpwd) };
    unsafe { env::set_var("PWD", normalize_path(&PathBuf::from(target))) };
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